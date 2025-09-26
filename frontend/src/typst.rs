use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use color_eyre::eyre::Result;
use common::typst_packages::TypstPackagePayload;
use futures_util::{SinkExt, StreamExt};
use gloo_worker::reactor::{ReactorScope, reactor};
use serde::{Deserialize, Serialize};
use tar::Archive;
use tracing::info;
use typst::diag::{
    FileError, FileResult, PackageError, PackageResult, Severity, SourceDiagnostic, SourceResult,
    Warned,
};
use typst::ecow::{EcoString, EcoVec};
use typst::foundations::{Bytes, Datetime, Dict, Str, Value};
use typst::layout::PagedDocument;
use typst::syntax::package::PackageSpec;
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World, WorldExt};
use typst_kit::fonts::{FontSearcher, FontSlot};
use typst_pdf::PdfOptions;
use web_sys::XmlHttpRequest;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct TypstCompilationMessage {
    pub is_fatal: bool,
    pub message: EcoString,
    pub hints: EcoVec<EcoString>,
    // TODO(veluca): handle warnings outside the main file.
    pub span: std::ops::Range<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct TypstCompiledDocument {
    pub svg_pages: Vec<String>,
    pub pdf: Vec<u8>,
}

impl Debug for TypstCompiledDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "document with {} pages, {} byte PDF",
            self.svg_pages.len(),
            self.pdf.len()
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TypstCompilationResult {
    pub document: Option<TypstCompiledDocument>,
    pub messages: EcoVec<TypstCompilationMessage>,
}

impl Default for TypstCompilationResult {
    fn default() -> Self {
        Self {
            document: None,
            messages: EcoVec::new(),
        }
    }
}

#[derive(Debug, Default)]
struct PackagesCache {
    packages: HashMap<PackageSpec, HashMap<PathBuf, Vec<u8>>>,
}

impl PackagesCache {
    fn get_package_files(
        &mut self,
        package: &PackageSpec,
    ) -> PackageResult<&HashMap<PathBuf, Vec<u8>>> {
        if !self.packages.contains_key(package) {
            let PackageSpec {
                namespace,
                name,
                version,
            } = package;

            let payload = TypstPackagePayload {
                namespace: namespace.into(),
                name: name.into(),
                version: version.to_string(),
            };

            let xhr = XmlHttpRequest::new().unwrap();
            xhr.open_with_async("POST", "/api/typst_packages", false)
                .unwrap();
            xhr.set_request_header("Content-Type", "application/json")
                .unwrap();
            let body = serde_json::to_string(&payload).unwrap();
            xhr.set_response_type(web_sys::XmlHttpRequestResponseType::Arraybuffer);
            xhr.send_with_opt_str(Some(&body)).unwrap();
            if xhr.status().unwrap() < 200 || xhr.status().unwrap() >= 300 {
                return Err(PackageError::NetworkFailed(Some(EcoString::from(format!(
                    "HTTP {}",
                    xhr.status().unwrap()
                )))));
            }
            let res = xhr.response().unwrap();
            let array: js_sys::Uint8Array = js_sys::Uint8Array::new(&res);
            let bytes = array.to_vec();

            let mut files = HashMap::new();
            let mut archive = Archive::new(bytes.as_slice());
            for entry in archive.entries().unwrap() {
                let mut entry = entry.unwrap();
                if !entry.header().entry_type().is_file() {
                    continue;
                }
                let path = entry.path().unwrap().to_path_buf();
                let path = path.strip_prefix("./").unwrap_or(&path).to_owned();
                let mut content = vec![];
                std::io::copy(&mut entry, &mut content).unwrap();
                files.insert(path, content);
            }

            info!("Loaded package {package:?} with {} files", files.len());

            self.packages.insert(package.clone(), files);
        }

        Ok(self.packages.get(package).unwrap())
    }
}

#[derive(Debug)]
pub struct TypstCompiler {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Arc<Vec<FontSlot>>,
    main: FileId,
    files: HashMap<PathBuf, Vec<u8>>,
    packages: Mutex<PackagesCache>,
}

impl TypstCompiler {
    fn new() -> TypstCompiler {
        // TODO(veluca): handle fonts better.
        let fonts = FontSearcher::new()
            .include_system_fonts(false)
            .include_embedded_fonts(true)
            .search();

        let mut inputs = Dict::new();
        inputs.insert(Str::from("gen_gen"), Value::Str(Str::from("GEN")));
        inputs.insert(
            Str::from("constraints_yaml"),
            Value::Str(Str::from("constraints.yaml")),
        );
        inputs.insert(
            Str::from("contest_yaml"),
            Value::Str(Str::from("../../contest.yaml")),
        );

        let library = Library::builder().with_inputs(inputs).build();

        TypstCompiler {
            library: LazyHash::new(library),
            book: LazyHash::new(fonts.book),
            fonts: Arc::new(fonts.fonts),
            main: FileId::new(None, VirtualPath::new("booklet.typ")),
            files: HashMap::new(),
            packages: Mutex::new(PackagesCache::default()),
        }
    }

    fn set_files(&mut self, files: HashMap<PathBuf, Vec<u8>>) {
        self.files = files;
    }

    /// Compile the Typst file.
    fn compile(&mut self) -> Result<TypstCompilationResult> {
        let mut messages = EcoVec::new();
        let mut add_errors = |msgs: &[SourceDiagnostic]| {
            for msg in msgs {
                let message = TypstCompilationMessage {
                    is_fatal: msg.severity == Severity::Error,
                    message: msg.message.clone(),
                    hints: msg.hints.clone(),
                    span: self.range(msg.span).unwrap_or(0..0),
                };
                messages.push(message);
            }
        };
        let Warned::<SourceResult<PagedDocument>> { output, warnings } = typst::compile(self);
        add_errors(&warnings);
        let document = match output {
            Ok(doc) => doc,
            Err(err) => {
                add_errors(&err);
                return Ok(TypstCompilationResult {
                    document: None,
                    messages,
                });
            }
        };

        let mut svg_pages = vec![];
        for page in document.pages.iter() {
            svg_pages.push(typst_svg::svg(page));
        }

        let pdf = match typst_pdf::pdf(&document, &PdfOptions::default()) {
            Ok(pdf) => pdf,
            Err(err) => {
                add_errors(&err);
                return Ok(TypstCompilationResult {
                    document: None,
                    messages,
                });
            }
        };
        Ok(TypstCompilationResult {
            document: Some(TypstCompiledDocument { svg_pages, pdf }),
            messages,
        })
    }

    fn get_file(&self, id: FileId) -> FileResult<Vec<u8>> {
        let mut packages = self.packages.try_lock().expect("lock poisoned");
        let file_store = if let Some(package) = id.package() {
            packages.get_package_files(package)?
        } else {
            &self.files
        };
        let path = id.vpath().as_rootless_path();
        let path = path.strip_prefix("./").unwrap_or(path);
        Ok(file_store
            .get(path)
            .ok_or(FileError::NotFound(path.to_owned()))?
            .to_owned())
    }
}

impl World for TypstCompiler {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts[index].get()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let bytes = self.get_file(id)?;
        Ok(Source::new(id, String::from_utf8(bytes.to_vec())?))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let bytes = self.get_file(id)?;
        Ok(Bytes::new(bytes.to_vec()))
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0).try_into().ok()?;
        let offset = time::UtcOffset::from_hms(offset, 0, 0).ok()?;
        time::OffsetDateTime::now_utc()
            .checked_to_offset(offset)
            .map(|time| Datetime::Date(time.date()))
    }
}

// TODO(veluca): better interface.
#[reactor]
pub async fn TypstWorker(
    mut scope: ReactorScope<HashMap<PathBuf, Vec<u8>>, TypstCompilationResult>,
) {
    let mut compiler = TypstCompiler::new();
    while let Some(files) = scope.next().await {
        compiler.set_files(files);
        let result = compiler.compile().unwrap();
        if scope.send(result).await.is_err() {
            break;
        }
    }
}

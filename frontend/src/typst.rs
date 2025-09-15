use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::Arc};

use color_eyre::eyre::Result;
use futures_util::{SinkExt, StreamExt};
use gloo_worker::reactor::{ReactorScope, reactor};
use serde::{Deserialize, Serialize};
use typst::{
    Library, World, WorldExt,
    diag::{
        FileError, FileResult, PackageError, PackageResult, Severity, SourceDiagnostic,
        SourceResult, Warned,
    },
    ecow::{EcoString, EcoVec},
    foundations::{Bytes, Datetime},
    layout::PagedDocument,
    syntax::{FileId, Source, VirtualPath, package::PackageSpec},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::fonts::{FontSearcher, FontSlot};
use typst_pdf::PdfOptions;

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

#[derive(Debug, Clone)]
pub struct TypstCompiler {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Arc<Vec<FontSlot>>,
    main: FileId,
    files: HashMap<PathBuf, Vec<u8>>,
}

impl TypstCompiler {
    fn new() -> TypstCompiler {
        // TODO(veluca): handle fonts better.
        let fonts = FontSearcher::new()
            .include_system_fonts(false)
            .include_embedded_fonts(true)
            .search();

        // TODO(veluca): pass inputs.
        let library = Library::builder().build();

        TypstCompiler {
            library: LazyHash::new(library),
            book: LazyHash::new(fonts.book),
            fonts: Arc::new(fonts.fonts),
            main: FileId::new(None, VirtualPath::new("statement.typ")),
            files: HashMap::new(),
        }
    }

    fn set_file_contents(&mut self, path: PathBuf, contents: Vec<u8>) {
        self.files.insert(path, contents);
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

    fn get_package(&self, package: &PackageSpec) -> PackageResult<&HashMap<PathBuf, Vec<u8>>> {
        // TODO(veluca): figure out how to retrieve packages in a web environment.
        PackageResult::Err(PackageError::NotFound(package.clone()))
    }

    fn get_file(&self, id: FileId) -> FileResult<&[u8]> {
        let file_store = if let Some(package) = id.package() {
            self.get_package(package)?
        } else {
            &self.files
        };
        let path = id.vpath().as_rootless_path();
        let path = path.strip_prefix("./").unwrap_or(path);
        Ok(file_store
            .get(path)
            .ok_or(FileError::NotFound(path.to_owned()))?)
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
pub async fn TypstWorker(mut scope: ReactorScope<Vec<u8>, TypstCompilationResult>) {
    let mut compiler = TypstCompiler::new();
    while let Some(doc) = scope.next().await {
        compiler.set_file_contents(PathBuf::new().join("statement.typ"), doc);
        let result = compiler.compile().unwrap();
        if scope.send(result).await.is_err() {
            break;
        }
    }
}

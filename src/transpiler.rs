use swc_common::errors::{ColorConfig, Handler};
use swc_core::common::comments::{Comments, SingleThreadedComments};
use swc_core::common::input::StringInput;
use swc_core::common::sync::Lrc;
use swc_core::common::{BytePos, FileName, Globals, LineCol, Mark, SourceMap, GLOBALS};
use swc_ecmascript::ast::EsVersion;
use swc_ecmascript::codegen::text_writer::JsWriter;
use swc_ecmascript::codegen::{Config as CodegenConfig, Emitter};
use swc_ecmascript::parser::lexer::Lexer;
use swc_ecmascript::parser::Syntax;
use swc_ecmascript::parser::{Capturing, Parser};
use swc_ecmascript::transforms::fixer::fixer;
use swc_ecmascript::transforms::hygiene::hygiene;
use swc_ecmascript::transforms::resolver;
use swc_ecmascript::transforms::typescript::strip;
use swc_ecmascript::visit::FoldWith;

use crate::errors::Error;

pub fn transpile_module(file_name: String, content: String) -> Result<String, Error> {
    let source_map: Lrc<SourceMap> = Default::default();
    let file: Lrc<swc_common::SourceFile> =
        source_map.new_source_file(FileName::Custom(file_name), content);
    let input = StringInput::from(&*file);

    let comments = SingleThreadedComments::default();
    let (handler, mut parser) = initialise_parser(source_map.clone(), &comments, input);

    let module = parser.parse_module().map_err(|e| {
        e.into_diagnostic(&handler).emit();
        Error::Parse
    })?;

    let globals = Globals::default();
    let module = GLOBALS.set(&globals, || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        let module = module.fold_with(&mut resolver(unresolved_mark, top_level_mark, true));
        let module = module.fold_with(&mut strip(top_level_mark));
        let module = module.fold_with(&mut hygiene());
        module.fold_with(&mut fixer(Some(&comments)))
    });

    let mut buffer = Vec::new();
    let mut mappings = Vec::new();

    let mut emitter = initialise_emitter(source_map, &comments, &mut buffer, &mut mappings);
    emitter.emit_module(&module).map_err(|_| Error::Emission)?;

    match String::from_utf8(buffer) {
        Ok(code) => Ok(code),
        Err(_) => Err(Error::UTF8InvalidSlice),
    }
}

fn initialise_parser<'a>(
    source_map: Lrc<SourceMap>,
    comments: &'a dyn Comments,
    input: StringInput<'a>,
) -> (Handler, Parser<Capturing<Lexer<'a>>>) {
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(source_map));
    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        EsVersion::Es2022,
        input,
        Some(comments),
    );
    let capturing = Capturing::new(lexer);
    let mut parser = Parser::new_from(capturing);

    for error in parser.take_errors() {
        error.into_diagnostic(&handler).emit();
    }

    (handler, parser)
}

fn initialise_emitter<'a>(
    source_map: Lrc<SourceMap>,
    comments: &'a dyn Comments,
    buffer: &'a mut Vec<u8>,
    mappings: &'a mut Vec<(BytePos, LineCol)>,
) -> Emitter<'a, JsWriter<'a, &'a mut Vec<u8>>, SourceMap> {
    Emitter {
        cfg: CodegenConfig {
            target: EsVersion::Es2022,
            ascii_only: false,
            minify: false,
            omit_last_semi: false,
        },
        cm: source_map.clone(),
        comments: Some(comments),
        wr: JsWriter::new(source_map, "\n", buffer, Some(mappings)),
    }
}

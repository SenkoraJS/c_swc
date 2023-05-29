use swc_core::common::comments::{Comments, SingleThreadedComments};
use swc_core::common::input::StringInput;
use swc_core::common::sync::Lrc;
use swc_core::common::{BytePos, FileName, Globals, LineCol, Mark, SourceMap, GLOBALS};
use swc_ecmascript::ast::EsVersion;
use swc_ecmascript::codegen::text_writer::JsWriter;
use swc_ecmascript::codegen::{Config as CodegenConfig, Emitter};
use swc_ecmascript::parser::lexer::Lexer;
use swc_ecmascript::parser::Parser;
use swc_ecmascript::parser::Syntax;
use swc_ecmascript::transforms::fixer::fixer;
use swc_ecmascript::transforms::hygiene::hygiene;
use swc_ecmascript::transforms::resolver;
use swc_ecmascript::transforms::typescript::strip;
use swc_ecmascript::visit::FoldWith;

fn main() {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom("test.js".into()),
        "function foo(test: number) { return test + 5 }".into(),
    );

    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        EsVersion::latest(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    let comments = SingleThreadedComments::default();
    let module = parser.parse_module().unwrap();

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

    let mut emitter = initialise_emitter(cm, &comments, &mut buffer, &mut mappings);
    emitter.emit_module(&module).unwrap();

    println!("Hello, world!");
    println!("{}", String::from_utf8(buffer).unwrap());
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

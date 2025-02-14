use rspack_core::{
  get_js_chunk_filename_template,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  runtime_globals::PUBLIC_PATH,
  ChunkUkey, Compilation, OutputOptions, PublicPath, RuntimeModule, SourceType,
};

use super::utils::get_undo_path;

#[derive(Debug, Default)]
pub struct PublicPathRuntimeModule {
  chunk: Option<ChunkUkey>,
}

impl PublicPathRuntimeModule {
  pub fn new() -> Self {
    Self {
      chunk: Default::default(),
    }
  }
}

impl RuntimeModule for PublicPathRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/public_path".to_string()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    match &compilation.options.output.public_path {
      PublicPath::String(str) => RawSource::from(
        include_str!("runtime/public_path.js").replace("__PUBLIC_PATH_PLACEHOLDER__", str),
      )
      .boxed(),
      PublicPath::Auto => {
        let chunk = compilation
          .chunk_by_ukey
          .get(&self.chunk.expect("The chunk should be attached."))
          .expect("Chunk is not found, make sure you had attach chunkUkey successfully.");
        let filename = get_js_chunk_filename_template(
          chunk,
          &compilation.options.output,
          &compilation.chunk_group_by_ukey,
        );
        let filename = filename.render_with_chunk(chunk, ".js", &SourceType::JavaScript);
        RawSource::from(auto_public_path_template(
          &filename,
          &compilation.options.output,
        ))
        .boxed()
      }
    }
  }
}

// TODO: should use `__webpack_require__.g`
const GLOBAL: &str = "self";

fn auto_public_path_template(filename: &str, output: &OutputOptions) -> String {
  let output_path = output.path.display().to_string();
  let undo_path = get_undo_path(filename, output_path, false);
  let assign = if undo_path.is_empty() {
    format!("{PUBLIC_PATH} = scriptUrl")
  } else {
    format!("{PUBLIC_PATH} = scriptUrl + '{undo_path}'")
  };
  format!(
    r#"
  var scriptUrl;
  if ({GLOBAL}.importScripts) scriptUrl = {GLOBAL}.location + "";
  var document = {GLOBAL}.document;
  if (!scriptUrl && document) {{
    if (document.currentScript) scriptUrl = document.currentScript.src;
      if (!scriptUrl) {{
        var scripts = document.getElementsByTagName("script");
		    if (scripts.length) scriptUrl = scripts[scripts.length - 1].src;
      }}
    }}
  // When supporting browsers where an automatic publicPath is not supported you must specify an output.publicPath manually via configuration",
  // or pass an empty string ("") and set the __webpack_public_path__ variable from your code to use your own logic.',
  if (!scriptUrl) throw new Error("Automatic publicPath is not supported in this browser");
  scriptUrl = scriptUrl.replace(/#.*$/, "").replace(/\?.*$/, "").replace(/\/[^\/]+$/, "/");
  {assign}
  "#
  )
}

#[test]
fn test_get_undo_path() {
  assert_eq!(get_undo_path("a", "/a/b/c".to_string(), true), "./");
  assert_eq!(
    get_undo_path("static/js/a.js", "/a/b/c".to_string(), false),
    "../../"
  );
}

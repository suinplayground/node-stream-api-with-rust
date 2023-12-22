use neon::prelude::*;

mod pass_through;
mod unbzip2;

#[neon::main]
// モジュールがロードされたときに1回呼び出されます
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // モジュールの一部として、Neon関数をエクスポートします
    cx.export_function("createPassThroughStream", pass_through::create_pass_through_stream)?;
    cx.export_function("handlePassThroughStreamChunk", pass_through::handle_pass_through_stream_chunk)?;
    cx.export_function("ungzip2Create", unbzip2::unbzip2_create)?;
    cx.export_function("ungzip2Chunk", unbzip2::unbzip2_chunk)?;
    cx.export_function("ungzip2Finish", unbzip2::unbzip2_finish)?;
    Ok(())
}

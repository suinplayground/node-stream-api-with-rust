use neon::prelude::*;

mod pass_through;

#[neon::main]
// モジュールがロードされたときに1回呼び出されます
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // モジュールの一部として、Neon関数をエクスポートします
    cx.export_function("createPassThroughStream", pass_through::create_pass_through_stream)?;
    cx.export_function("handlePassThroughStreamChunk", pass_through::handle_pass_through_stream_chunk)?;
    Ok(())
}

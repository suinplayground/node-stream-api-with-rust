// 型定義ファイルは自分で書きましょう

declare class PassThroughStream {
  #private;
}

export declare const createPassThroughStream: () => PassThroughStream;

export declare const handlePassThroughStreamChunk: (
  stream: PassThroughStream,
  encoding: string,
  chunk: Uint8Array,
) => Promise<Uint8Array>;

import type { Backend } from "./backend";
import { createWebGl2 } from "./webgl2";
import { createWebGpu } from "./webgpu";

export { type Backend, fetchLinear, type LinearImage } from "./backend";
export {
  drawnSize,
  type Geometry,
  geometryKey,
  geometryOf,
  outputSize,
  type Placement,
  sourceRegion,
} from "./geometry";
export { createWebGl2 } from "./webgl2";
export { createWebGpu } from "./webgpu";

/**
 * The preview for a canvas: WebGPU where the web view has it, WebGL2
 * otherwise, or always with `forceWebgl2` (`SEURA_PREVIEW=webgl2`).
 */
export async function createPreview(
  canvas: HTMLCanvasElement,
  forceWebgl2: boolean,
): Promise<Backend | null> {
  if (!forceWebgl2) {
    const webgpu = await createWebGpu(canvas);
    if (webgpu) return webgpu;
  }
  return createWebGl2(canvas);
}

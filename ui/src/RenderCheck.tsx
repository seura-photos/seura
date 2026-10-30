import { useEffect, useRef, useState } from "react";
import { commands, type RenderSettings } from "./bindings";
import { t } from "./messages";
import { type Backend, createWebGl2, createWebGpu, fetchLinear } from "./preview";

/**
 * The render check (`SEURA_RENDER_CHECK=1`; M1 slice 5): renders the core's
 * reference image with each preview path this web view has, sends the
 * pixels to the core to compare with export, then the app exits with the
 * result. CI runs it on every platform.
 */
export function RenderCheck({ settings }: { settings: RenderSettings }) {
  const [log, setLog] = useState<string[]>([]);
  // Effects run twice in development; the check runs once.
  const started = useRef(false);

  useEffect(() => {
    if (started.current) return;
    started.current = true;
    const run = async () => {
      const skipped: string[] = [];
      const problems: string[] = [];
      const [image, guide] = await Promise.all([
        fetchLinear("check-image"),
        fetchLinear("check-guide"),
      ]);
      const paths: [string, () => Promise<Backend | null>][] = [
        ["webgpu", () => createWebGpu(null)],
        ["webgl2", () => Promise.resolve(createWebGl2(document.createElement("canvas")))],
      ];
      for (const [name, create] of paths) {
        try {
          const backend = await create();
          if (!backend) {
            skipped.push(name);
            continue;
          }
          backend.upload(image, guide);
          const pixels = await backend.read(settings);
          backend.destroy();
          const result = await commands.checkRender(
            name,
            image.width,
            image.height,
            Array.from(pixels),
          );
          const line =
            result.status === "ok" ? `${name}: ${String(result.data)}` : `${name}: ${result.error}`;
          if (result.status === "error") problems.push(line);
          setLog((previous) => [...previous, line]);
        } catch (error) {
          problems.push(`${name}: ${String(error)}`);
        }
      }
      if (skipped.length === paths.length) problems.push("no preview path is available");
      await commands.finishRenderCheck(skipped, problems);
    };
    void run().catch((error: unknown) => commands.finishRenderCheck([], [String(error)]));
  }, [settings]);

  return (
    <main className="render-check">
      <p>{t("check-running")}</p>
      <ul>
        {log.map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ul>
    </main>
  );
}

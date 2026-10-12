import { describe, it, expect, vi } from "vitest";
import { FlyoutMotion } from "../app/flyoutMotion";
const panel = () => ({
  dataset: {} as Record<string, string>,
  style: { transform: "" },
  animate: vi.fn(
    (_keyframes: Keyframe[], _options: KeyframeAnimationOptions) => ({
      cancel: vi.fn(),
      finished: Promise.resolve(),
    }),
  ),
});
describe("flyout motion", () => {
  it("ignores a stale closing event and startup snapshot after a newer opening", async () => {
    const el = panel(),
      send = vi.fn(async () => true);
    const motion = new FlyoutMotion(
      el as unknown as HTMLElement,
      send,
      () => true,
    );
    await motion.open(3);
    await motion.close(2);
    await motion.open(1);
    expect(el.dataset.motion).toBe("visible");
    expect(send).toHaveBeenCalledTimes(1);
  });
  it("does not hide a window that reopened while its exit animation was pending", async () => {
    const el = panel(),
      send = vi.fn(async () => true);
    const motion = new FlyoutMotion(
      el as unknown as HTMLElement,
      send,
      () => false,
    );
    await motion.open(1);
    let reject!: (reason?: unknown) => void;
    const finished = new Promise<void>((_resolve, fail) => {
      reject = fail;
    });
    const cancelled = vi.fn(() => reject(new Error("cancelled")));
    el.animate.mockImplementationOnce(() => ({ cancel: cancelled, finished }));
    const closing = motion.close(2);
    await motion.open(3);
    await closing;
    expect(cancelled).toHaveBeenCalledOnce();
    expect(send).not.toHaveBeenCalledWith("finish_hide", { token: 2 });
    expect(el.dataset.motion).toBe("visible");
  });
  it("prepares the panel offscreen before showing the native window, then slides from the right", async () => {
    const el = panel();
    const send = vi.fn(async () => {
      expect(el.style.transform).toContain("100%");
      return true;
    });
    await new FlyoutMotion(
      el as unknown as HTMLElement,
      send,
      () => false,
    ).open(4);
    expect(send).toHaveBeenCalledWith("show_flyout_ready", { token: 4 });
    expect(el.animate).toHaveBeenCalledOnce();
    expect(el.animate.mock.calls[0][0]).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ transform: "translateX(0)" }),
      ]),
    );
  });
  it("respects reduced motion and still completes native hide", async () => {
    const el = panel(),
      send = vi.fn(async () => true);
    const motion = new FlyoutMotion(
      el as unknown as HTMLElement,
      send,
      () => true,
    );
    await motion.open(1);
    await motion.close(2);
    expect(el.animate).not.toHaveBeenCalled();
    expect(send).toHaveBeenCalledWith("finish_hide", { token: 2 });
  });
});

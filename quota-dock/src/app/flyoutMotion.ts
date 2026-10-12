export type MotionSender = (
  command: string,
  args: { token: number },
) => Promise<boolean | void>;
export class FlyoutMotion {
  private revision = 0;
  private animation?: Animation;
  private latestToken?: number;
  private accept(token: number) {
    if (this.latestToken !== undefined) {
      const distance = (token - this.latestToken) >>> 0;
      if (distance === 0 || distance >= 0x80000000) return false;
    }
    this.latestToken = token;
    return true;
  }
  constructor(
    private element: HTMLElement,
    private send: MotionSender,
    private reduced: () => boolean,
  ) {}
  private stop() {
    this.animation?.cancel();
    this.animation = undefined;
  }
  private currentTransform() {
    return typeof getComputedStyle === "function"
      ? getComputedStyle(this.element).transform
      : "translateX(0)";
  }
  async open(token: number) {
    if (!this.accept(token)) return;
    const revision = ++this.revision;
    const from = ["opening", "closing"].includes(
      this.element.dataset.motion || "",
    )
      ? this.currentTransform()
      : "translateX(calc(100% + 16px))";
    this.stop();
    this.element.dataset.motion = "opening";
    this.element.style.transform = from;
    const accepted = await this.send("show_flyout_ready", { token });
    if (revision !== this.revision || accepted === false) return;
    if (!this.reduced()) {
      this.animation = this.element.animate(
        [{ transform: from }, { transform: "translateX(0)" }],
        { duration: 230, easing: "cubic-bezier(.22, 1, .36, 1)" },
      );
      try {
        await this.animation.finished;
      } catch {
        return;
      }
    }
    if (revision === this.revision) {
      this.element.style.transform = "translateX(0)";
      this.element.dataset.motion = "visible";
      this.animation = undefined;
    }
  }
  async close(token: number) {
    if (!this.accept(token)) return;
    const revision = ++this.revision;
    const from = this.currentTransform();
    this.stop();
    this.element.dataset.motion = "closing";
    this.element.style.transform = from;
    if (!this.reduced()) {
      this.animation = this.element.animate(
        [{ transform: from }, { transform: "translateX(calc(100% + 16px))" }],
        { duration: 170, easing: "cubic-bezier(.4, 0, 1, 1)" },
      );
      try {
        await this.animation.finished;
      } catch {
        return;
      }
    }
    if (revision !== this.revision) return;
    this.element.dataset.motion = "hidden";
    this.element.style.transform = "translateX(calc(100% + 16px))";
    this.animation = undefined;
    await this.send("finish_hide", { token });
  }
  cancel() {
    ++this.revision;
    this.stop();
    this.element.dataset.motion = "visible";
    this.element.style.transform = "translateX(0)";
  }
}

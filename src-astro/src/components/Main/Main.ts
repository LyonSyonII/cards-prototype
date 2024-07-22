import { invoke } from "@tauri-apps/api/core";
import { emit, listen, once } from "@tauri-apps/api/event";

type ConnectionMode = "serve" | "connect";

export class Main extends HTMLElement {
  private readonly output: HTMLPreElement;
  private readonly connectInp: HTMLInputElement;
  private readonly connectBtn: HTMLButtonElement;
  private readonly serveBtn: HTMLButtonElement;
  private readonly input: HTMLInputElement;
  private mode: ConnectionMode | undefined;

  constructor() {
    super();
    this.output = this.querySelector<HTMLPreElement>("#output")!;
    this.connectInp = this.querySelector<HTMLInputElement>("#connect-in")!;
    this.connectBtn = this.querySelector<HTMLButtonElement>("#connect-btn")!;
    this.serveBtn = this.querySelector<HTMLButtonElement>("#serve-btn")!;
    this.input = this.querySelector<HTMLInputElement>("#input")!;

    this.connectInp.addEventListener(
      "keyup",
      ({ key }) => key === "Enter" && this.connect(),
    );
    this.connectBtn.addEventListener("click", async () => this.connect());
    this.serveBtn.addEventListener("click", async () => this.serve());
    this.input.addEventListener("keyup", async (event) => {
      if (event.key !== "Enter") return;

      event.preventDefault();
      this.print(this.input.value);
      await emit("SEND", this.input.value);
    });
    listen("RECEIVE", async ({ payload }: { payload: string }) => {
      this.print(payload, { received: true });
    });
  }

  private async connect() {
    const addr = this.connectInp.value;
    try {
      await invoke("connect", { addr });
      this.mode = "connect";

      this.print(`Connected with ${addr}`);
      hideElements(this.connectInp, this.connectBtn, this.serveBtn);
      showElements(this.input);
    } catch (error: any) {
      this.print(error, { mode: "connect"} );
    }
  }

  private async serve() {
    try {
      const addr = await invoke<string>("serve");
      this.mode = "serve";

      this.print(`Listening on ${addr}`);
      hideElements(this.connectInp, this.connectBtn, this.serveBtn);
      await once<string>("RECEIVE", ({ payload }) => {
        this.print(`Connected with ${payload}`);
        showElements(this.input);
      });
    } catch (error: any) {
      this.print(error, { mode: "serve" });
      return;
    }
  }
  
  private print(s: string, { mode, received }: { mode?: ConnectionMode, received?: boolean } = { received: false }) {
    mode = this.mode || mode;
    if (!mode) {
      return;
    }
    if (received) {
      mode = mode === "connect" ? "serve" : "connect";
    }
    mode && (this.output.textContent += `[${mode}] ${s}\n`);
  }
}

function showElements(...nodes: HTMLElement[]) {
  for (const n of nodes) {
    n.style.display = "block";
  }
}

function hideElements(...nodes: HTMLElement[]) {
  for (const n of nodes) {
    n.style.display = "none";
  }
}

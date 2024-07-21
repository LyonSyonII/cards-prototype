import { invoke } from "@tauri-apps/api/core";
import { listen, once } from "@tauri-apps/api/event";

export async function main() {
  await connect();
  await serve();
}

async function connect() {
  const output = document.querySelector<HTMLParagraphElement>("#output")!;
  const input = document.querySelector<HTMLInputElement>("#connect-in")!;
  const println = (s: string) => {
    output.textContent += `[connect] ${s}\n`;
  }

  document.querySelector<HTMLButtonElement>("#connect-btn")!.addEventListener("click", async () => {
    const addr = input.value;
    await invoke("connect", { addr })
      .then(() => println(`Connected with ${addr}`))
      .catch(e => println(e));
  });
  // await listen<string>("RECEIVE", ({ payload }) => println(payload));
}

async function serve() {
  const output = document.querySelector<HTMLParagraphElement>("#output")!;
  const println = (s: string) => {
    output.textContent += `[serve] ${s}\n`;
  }

  document.querySelector<HTMLButtonElement>("#serve-btn")!.addEventListener("click", async () => { 
    invoke<string>("serve")
      .then(addr => println(`Listening on ${addr}`))
      .catch(e => println(e));
    await once<string>("RECEIVE", ({ payload }) => println(`Connected with ${payload}`))
  });
}

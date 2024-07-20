import { invoke } from "@tauri-apps/api/core";

export function main() {
  const greetInput = document.querySelector<HTMLInputElement>("#greet-in")!;
  const greetOutput = document.querySelector<HTMLParagraphElement>("#greet-out")!;
  document.querySelector<HTMLButtonElement>("button")!.addEventListener("click", async () => {
    greetOutput.textContent = await invoke("greet", { name: greetInput.value });
  });
}

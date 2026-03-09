import "./index.css";
import { compile_run } from "../../pkg";

const inp = document.getElementById("input") as HTMLTextAreaElement;
const out = document.getElementById("output") as HTMLTextAreaElement;
const stat = document.getElementById("status") as HTMLParagraphElement;
const btn = document.getElementById("run") as HTMLButtonElement;

// const bf = init();

btn.addEventListener("click", async () => {
  out.value = "";

//   await bf;

  // TODO: Stdin
  compile_run(
    inp.value,
    () => 0,
    (v: number) => {
      out.value += String.fromCharCode(v);
      return undefined;
    },
    (n: BigInt) => {
      const s = Number(n);

      stat.innerText = `Executed in ${s.toFixed(2)} ms`;

      return undefined;
    },
  );
});

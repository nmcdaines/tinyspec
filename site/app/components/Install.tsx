"use client";

import { useEffect, useState } from "react";

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={handleCopy}
      className="text-muted transition-colors hover:text-foreground"
      aria-label="Copy to clipboard"
    >
      {copied ? (
        <svg
          className="h-4 w-4"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="m4.5 12.75 6 6 9-13.5"
          />
        </svg>
      ) : (
        <svg
          className="h-4 w-4"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 0 1-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 0 1 1.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 0 0-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 0 1-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 0 0-3.375-3.375h-1.5a1.125 1.125 0 0 1-1.125-1.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H9.75"
          />
        </svg>
      )}
    </button>
  );
}

function TerminalBlock({ command }: { command: string }) {
  return (
    <div className="flex items-center justify-between gap-4 rounded-lg bg-code-bg px-5 py-3 font-mono text-sm">
      <div>
        <span className="text-muted select-none">$ </span>
        <code>{command}</code>
      </div>
      <CopyButton text={command} />
    </div>
  );
}

function ClaudeCodeBlock({ command }: { command: string }) {
  return (
    <div className="overflow-hidden rounded-lg border border-border">
      <div className="flex items-center gap-2 border-b border-border bg-code-bg px-4 py-2">
        <div className="flex gap-1.5">
          <div className="h-2.5 w-2.5 rounded-full bg-[#ff5f57]" />
          <div className="h-2.5 w-2.5 rounded-full bg-[#febc2e]" />
          <div className="h-2.5 w-2.5 rounded-full bg-[#28c840]" />
        </div>
        <span className="ml-2 text-xs text-muted">Claude Code</span>
      </div>
      <div className="flex items-center justify-between gap-4 bg-[#1e1e1e] px-5 py-3 font-mono text-sm text-[#d4d4d4]">
        <div>
          <span className="text-[#569cd6] select-none">&gt; </span>
          <code>{command}</code>
        </div>
        <CopyButton text={command} />
      </div>
    </div>
  );
}

type Step = {
  step: string;
  title: string;
  command: string;
  type: "terminal" | "claude";
};

const stepsAfterInstall: Step[] = [
  {
    step: "2",
    title: "Initialize in your project",
    command: "tinyspec init",
    type: "terminal",
  },
  {
    step: "3",
    title: "Create a new spec",
    command: "tinyspec new my-feature",
    type: "terminal",
  },
  {
    step: "4",
    title: "Refine with Claude",
    command: "/tinyspec:refine my-feature",
    type: "claude",
  },
  {
    step: "5",
    title: "Implement the plan",
    command: "/tinyspec:do my-feature",
    type: "claude",
  },
];

type InstallMethod = "quick" | "cargo";

const installCommands: Record<InstallMethod, string> = {
  quick: "curl -fsSL https://tinyspec.dev/install.sh | sh",
  cargo: "cargo install tinyspec",
};

const windowsInstallCommands: Record<InstallMethod, string> = {
  quick: "irm https://tinyspec.dev/install.ps1 | iex",
  cargo: "cargo install tinyspec",
};

function useIsWindows() {
  const [isWindows, setIsWindows] = useState(false);
  useEffect(() => {
    setIsWindows(navigator.platform?.startsWith("Win") ?? false);
  }, []);
  return isWindows;
}

export default function Install() {
  const [method, setMethod] = useState<InstallMethod>("quick");
  const isWindows = useIsWindows();
  const commands = isWindows ? windowsInstallCommands : installCommands;

  return (
    <section className="px-6 py-24">
      <div className="mx-auto max-w-2xl">
        <h2 className="text-center text-3xl font-bold tracking-tight sm:text-4xl">
          Get started in minutes
        </h2>
        <p className="mx-auto mt-4 max-w-lg text-center text-muted">
          Install tinyspec, create a spec, and let Claude Code handle the rest.
        </p>
        <div className="mt-12 space-y-6">
          <div className="flex gap-4">
            <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-accent text-sm font-bold text-white">
              1
            </div>
            <div className="flex-1">
              <div className="mb-2 flex items-center gap-3">
                <p className="font-medium">Install tinyspec</p>
                <div className="flex gap-1 rounded-md bg-code-bg p-0.5 text-xs">
                  <button
                    onClick={() => setMethod("quick")}
                    className={`rounded px-2 py-0.5 transition-colors ${
                      method === "quick"
                        ? "bg-accent text-white"
                        : "text-muted hover:text-foreground"
                    }`}
                  >
                    Quick install
                  </button>
                  <button
                    onClick={() => setMethod("cargo")}
                    className={`rounded px-2 py-0.5 transition-colors ${
                      method === "cargo"
                        ? "bg-accent text-white"
                        : "text-muted hover:text-foreground"
                    }`}
                  >
                    From source
                  </button>
                </div>
              </div>
              <TerminalBlock command={commands[method]} />
            </div>
          </div>
          {stepsAfterInstall.map((item) => (
            <div key={item.step} className="flex gap-4">
              <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-accent text-sm font-bold text-white">
                {item.step}
              </div>
              <div className="flex-1">
                <p className="mb-2 font-medium">{item.title}</p>
                {item.type === "claude" ? (
                  <ClaudeCodeBlock command={item.command} />
                ) : (
                  <TerminalBlock command={item.command} />
                )}
              </div>
            </div>
          ))}
        </div>
        <p className="mt-10 text-center text-sm text-muted">
          Need to adjust the plan? Run{" "}
          <code className="rounded bg-code-bg px-1.5 py-0.5 font-mono text-xs">
            /tinyspec:refine
          </code>{" "}
          again to iterate, then{" "}
          <code className="rounded bg-code-bg px-1.5 py-0.5 font-mono text-xs">
            /tinyspec:do
          </code>{" "}
          to continue implementing.
        </p>
      </div>
    </section>
  );
}

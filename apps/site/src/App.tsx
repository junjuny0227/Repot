import { useEffect, useState } from "react";
import { detectPlatform, macArchPreference, type NavigatorLike, type Platform } from "./downloads";
import { Header } from "./sections/Header";
import { Hero } from "./sections/Hero";
import { HowItWorks } from "./sections/HowItWorks";
import { Principles } from "./sections/Principles";
import { Tools } from "./sections/Tools";
import { Install } from "./sections/Install";
import { Footer } from "./sections/Footer";

type HintNavigator = NavigatorLike & {
  userAgentData?: NavigatorLike["userAgentData"] & {
    getHighEntropyValues?: (hints: string[]) => Promise<{ architecture?: string }>;
  };
};

export default function App({ nav = globalThis.navigator as HintNavigator | undefined }: { nav?: HintNavigator }) {
  const [platform] = useState<Platform>(() => (nav ? detectPlatform(nav) : "unknown"));
  const [macArch, setMacArch] = useState<"mac-arm" | "mac-intel">("mac-arm");

  useEffect(() => {
    if (platform !== "mac") return;
    let live = true;
    nav?.userAgentData?.getHighEntropyValues?.(["architecture"])
      .then((v) => live && setMacArch(macArchPreference(v.architecture)))
      .catch(() => {}); // The hint is optional; keep the Apple silicon default.
    return () => { live = false; };
  }, [nav, platform]);

  return (
    <>
      <Header />
      <main id="main">
        <Hero platform={platform} macArch={macArch} />
        <HowItWorks />
        <Principles />
        <Tools />
        <Install platform={platform} />
      </main>
      <Footer />
    </>
  );
}

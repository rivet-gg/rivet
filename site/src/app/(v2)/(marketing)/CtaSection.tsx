"use client";

import Link from "next/link";
import { Button } from "@rivet-gg/components";
import { Icon, faArrowRight } from "@rivet-gg/icons";
import { MarketingButton } from "./MarketingButton";

// CTA section
export const CtaSection = () => {
  return (
    <div className="mx-auto max-w-7xl px-6 py-36 lg:py-52 lg:px-8 border-t border-white/5 mt-24">
      <div className="text-center">
        <h2 className="text-4xl font-medium tracking-tight text-white">Get building today</h2>
        <p className="mt-6 text-xl text-white/70 max-w-2xl mx-auto">
          Start for free, no credit card required. Deploy your first serverless project in minutes.
        </p>
        
        <div className="mt-12 flex items-center justify-center gap-x-6">
          <MarketingButton href="#deploy" primary>
            Deploy Now
          </MarketingButton>
          <MarketingButton href="/docs/rivet-vs-cloudflare-workers">
            <span>On-Prem Cloudflare Workers</span>
            <Icon
              icon={faArrowRight}
              className="ml-2 text-xs group-hover:translate-x-0.5 transition-transform"
            />
          </MarketingButton>
          {/*<MarketingButton href="#demo">
            <span>Book Demo</span>
          </MarketingButton>*/}
        </div>
      </div>
    </div>
  );
};

import type { IconProps } from "@/components/ui/floating-icons-hero-section";
import {
  Cloud,
  Cpu,
  Database,
  GitBranch,
  Box,
  Layers,
  Server,
  Workflow,
  Boxes,
  Network,
  Code2,
  Terminal,
  Sparkles,
  Container,
  Globe,
  Zap,
} from "lucide-react";

const icons: IconProps[] = [
  { id: 1, icon: Cloud, className: "top-[10%] left-[10%]" },
  { id: 2, icon: Server, className: "top-[20%] right-[8%]" },
  { id: 3, icon: Cpu, className: "top-[80%] left-[10%]" },
  { id: 4, icon: Database, className: "bottom-[10%] right-[10%]" },
  { id: 5, icon: GitBranch, className: "top-[5%] left-[30%]" },
  { id: 6, icon: Layers, className: "top-[5%] right-[30%]" },
  { id: 7, icon: Workflow, className: "bottom-[8%] left-[25%]" },
  { id: 8, icon: Box, className: "top-[40%] left-[15%]" },
  { id: 9, icon: Boxes, className: "top-[75%] right-[25%]" },
  { id: 10, icon: Network, className: "top-[90%] left-[70%]" },
  { id: 11, icon: Code2, className: "top-[50%] right-[5%]" },
  { id: 12, icon: Terminal, className: "top-[55%] left-[5%]" },
  { id: 13, icon: Sparkles, className: "top-[5%] left-[55%]" },
  { id: 14, icon: Container, className: "bottom-[5%] right-[45%]" },
  { id: 15, icon: Globe, className: "top-[25%] right-[20%]" },
  { id: 16, icon: Zap, className: "top-[60%] left-[30%]" },
];

export const heroFloatingIcons = icons;

"use client";

import { useReducedMotion } from "@/components/motion";

/**
 * Hero background: minimal knowledge graph (nodes + edges).
 * Foreground at 4–8% opacity; no accent. Suggests "landscape as graph."
 * Optional slow drift when reduced-motion is off.
 */
export function KnowledgeGraphMesh({ className = "" }: { className?: string }) {
  const reduced = useReducedMotion();

  // Fixed deterministic layout: sparse network, ~50 nodes, clustered
  const width = 1200;
  const height = 800;
  const nodeR = 2;
  const nodes: [number, number][] = [
    [120, 200],
    [280, 180],
    [260, 320],
    [400, 240],
    [380, 400],
    [520, 160],
    [500, 300],
    [640, 220],
    [620, 380],
    [760, 260],
    [740, 420],
    [900, 200],
    [880, 360],
    [1000, 280],
    [180, 420],
    [320, 480],
    [460, 520],
    [580, 460],
    [700, 500],
    [820, 480],
    [940, 440],
    [200, 560],
    [360, 600],
    [500, 640],
    [660, 580],
    [800, 560],
    [920, 520],
    [240, 680],
    [420, 700],
    [600, 660],
    [760, 620],
    [180, 100],
    [340, 80],
    [500, 120],
    [660, 100],
    [820, 140],
    [980, 120],
    [100, 340],
    [920, 200],
    [1080, 320],
    [1100, 480],
    [80, 520],
    [1140, 600],
    [60, 660],
    [200, 740],
    [380, 760],
    [560, 720],
    [720, 680],
    [880, 640],
  ];
  const edges: [number, number][] = [
    [0, 1],
    [1, 2],
    [1, 3],
    [2, 4],
    [3, 4],
    [3, 5],
    [4, 6],
    [5, 6],
    [5, 7],
    [6, 8],
    [7, 8],
    [7, 9],
    [8, 10],
    [9, 10],
    [9, 11],
    [10, 12],
    [11, 12],
    [11, 13],
    [14, 15],
    [15, 16],
    [16, 17],
    [17, 18],
    [18, 19],
    [19, 20],
    [14, 21],
    [16, 22],
    [18, 23],
    [20, 24],
    [21, 25],
    [22, 26],
    [23, 27],
    [25, 28],
    [26, 29],
    [27, 30],
    [28, 31],
    [29, 32],
    [30, 33],
    [31, 32],
    [32, 33],
    [34, 35],
    [35, 36],
    [36, 37],
    [37, 38],
    [0, 14],
    [13, 20],
    [24, 39],
    [40, 41],
    [42, 43],
    [44, 45],
    [46, 47],
  ];

  return (
    <svg
      className={`pointer-events-none absolute inset-0 h-full w-full ${reduced ? "" : "mesh-drift"} ${className}`}
      style={{ animationDuration: "90s" }}
      aria-hidden
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="xMidYMid slice"
    >
      <g className="fill-none stroke-foreground" style={{ strokeOpacity: 0.06 }}>
        {edges.map(([a, b], i) => {
          const from = nodes[a];
          const to = nodes[b];
          if (!from || !to) return null;
          const [x1, y1] = from;
          const [x2, y2] = to;
          return (
            <line
              key={i}
              x1={x1}
              y1={y1}
              x2={x2}
              y2={y2}
              strokeWidth={1}
            />
          );
        })}
      </g>
      <g className="fill-foreground" style={{ fillOpacity: 0.06 }}>
        {nodes.map(([x, y], i) => (
          <circle key={i} cx={x} cy={y} r={nodeR} />
        ))}
      </g>
    </svg>
  );
}

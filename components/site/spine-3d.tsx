"use client";

// A procedurally modeled human spine (7 cervical, 12 thoracic, 5 lumbar vertebrae + sacrum),
// built from Three.js primitives and rotating slowly in real time. No external model or texture
// files, so it loads instantly and respects the site's strict Content-Security-Policy.

import { Canvas, useFrame } from "@react-three/fiber";
import { useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";

const BONE = "#efe6d6";
const DISC = "#9fbfd6";

// Sagittal S-curve, cervical on the left, sacrum on the right; +y points posterior.
const CURVE = new THREE.CatmullRomCurve3([
  new THREE.Vector3(-5.1, 0.15, 0),
  new THREE.Vector3(-4.1, -0.12, 0),
  new THREE.Vector3(-2.6, 0.22, 0),
  new THREE.Vector3(-0.6, 0.48, 0),
  new THREE.Vector3(1.4, 0.32, 0),
  new THREE.Vector3(2.9, -0.08, 0),
  new THREE.Vector3(4.1, -0.12, 0),
  new THREE.Vector3(4.9, 0.25, 0),
]);

interface Level {
  u: number;
  r: number; // vertebral body radius
  h: number; // vertebral body height (along the spine)
  spine: number; // spinous process length
  tilt: number; // caudal tilt of the spinous process
  trans: number; // transverse process length
}

function buildLevels(): Level[] {
  const count = 24;
  const raw = Array.from({ length: count }, (_, i) => {
    const k = i / (count - 1);
    const region = i < 7 ? "c" : i < 19 ? "t" : "l";
    return {
      k,
      r: 0.2 + k * 0.2,
      h: 0.2 + k * 0.17,
      spine: region === "c" ? 0.22 + k * 0.5 : region === "t" ? 0.5 : 0.4,
      tilt: region === "c" ? 0.2 : region === "t" ? 0.8 : 0.35,
      trans: region === "c" ? 0.16 : region === "t" ? 0.26 : 0.36,
    };
  });
  // Space levels along the curve by their own size (+ a disc gap), so larger lumbar levels get more room.
  const L = CURVE.getLength();
  const steps = raw.map((v) => v.h * 1.32);
  const total = steps.reduce((x, y) => x + y, 0);
  const span = 0.84 * L;
  const scale = span / total;
  let acc = 0.03 * L;
  return raw.map((v, i) => {
    const center = acc + (steps[i] * scale) / 2;
    acc += steps[i] * scale;
    return { u: center / L, r: v.r, h: v.h, spine: v.spine, tilt: v.tilt, trans: v.trans };
  });
}

function bodyGeometry(r: number, h: number) {
  // Waisted cylinder with rounded rims, revolved around Y.
  const pts: THREE.Vector2[] = [];
  const half = h / 2;
  const steps = 10;
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const y = -half + t * h;
    const waist = 1 - 0.12 * Math.sin(Math.PI * t);
    const rim = Math.min(1, Math.sin(Math.PI * Math.min(t, 1 - t) * 2.2 + 0.35));
    pts.push(new THREE.Vector2(r * waist * (0.86 + 0.14 * rim), y));
  }
  pts.unshift(new THREE.Vector2(0, -half));
  pts.push(new THREE.Vector2(0, half));
  return new THREE.LatheGeometry(pts, 28);
}

function Vertebra({ level, bone }: { level: Level; bone: THREE.Material }) {
  const { r, h } = level;
  const ringR = r * 0.72;
  const tube = r * 0.2;
  const canalY = r + ringR * 0.75;
  const geo = useMemo(() => bodyGeometry(r, h), [r, h]);
  return (
    <group>
      {/* Body: lathe axis rotated from Y to X (along the spine) */}
      <mesh geometry={geo} material={bone} rotation={[0, 0, Math.PI / 2]} />
      {/* Neural arch around the spinal canal */}
      <mesh material={bone} position={[0, canalY, 0]} rotation={[0, Math.PI / 2, 0]}>
        <torusGeometry args={[ringR, tube, 10, 20, Math.PI]} />
      </mesh>
      {/* Pedicles joining arch to body */}
      {[-1, 1].map((s) => (
        <mesh key={s} material={bone} position={[0, r * 0.9, s * ringR]}>
          <cylinderGeometry args={[tube, tube * 1.1, r * 0.8, 10]} />
        </mesh>
      ))}
      {/* Spinous process, tilted toward the sacrum */}
      <group position={[0, canalY + ringR, 0]} rotation={[0, 0, -level.tilt]}>
        <mesh material={bone} position={[0, level.spine / 2, 0]}>
          <capsuleGeometry args={[tube * 0.95, level.spine, 6, 10]} />
        </mesh>
      </group>
      {/* Transverse processes */}
      {[-1, 1].map((s) => (
        <group key={`t${s}`} position={[0, canalY * 0.85, s * ringR]} rotation={[s * (Math.PI / 2 - 0.25), 0, 0]}>
          <mesh material={bone} position={[0, level.trans / 2, 0]}>
            <capsuleGeometry args={[tube * 0.75, level.trans, 6, 10]} />
          </mesh>
        </group>
      ))}
    </group>
  );
}

function Spine({ animate }: { animate: boolean }) {
  const roll = useRef<THREE.Group>(null);
  const float = useRef<THREE.Group>(null);
  const levels = useMemo(buildLevels, []);
  const bone = useMemo(() => new THREE.MeshStandardMaterial({ color: BONE, roughness: 0.5, metalness: 0.02 }), []);
  const disc = useMemo(() => new THREE.MeshStandardMaterial({ color: DISC, roughness: 0.35, metalness: 0.05 }), []);

  const placed = useMemo(() => {
    const xAxis = new THREE.Vector3(1, 0, 0);
    return levels.map((lv) => {
      const p = CURVE.getPointAt(lv.u);
      const t = CURVE.getTangentAt(lv.u);
      const q = new THREE.Quaternion().setFromUnitVectors(xAxis, t);
      return { lv, p, q };
    });
  }, [levels]);

  const discs = useMemo(() => {
    const xAxis = new THREE.Vector3(1, 0, 0);
    return placed.slice(0, -1).map((a, i) => {
      const b = placed[i + 1];
      const u = (a.lv.u + b.lv.u) / 2;
      const p = CURVE.getPointAt(u);
      const q = new THREE.Quaternion().setFromUnitVectors(xAxis, CURVE.getTangentAt(u));
      const r = (a.lv.r + b.lv.r) / 2;
      const gap = a.p.distanceTo(b.p) - (a.lv.h + b.lv.h) / 2;
      return { p, q, r, len: Math.max(0.03, gap * 0.92) };
    });
  }, [placed]);

  const sacrum = useMemo(() => {
    const xAxis = new THREE.Vector3(1, 0, 0);
    const last = levels[levels.length - 1];
    const start = last.u + 0.025;
    const segs = 8; // 5 sacral + 3 coccygeal
    return Array.from({ length: segs }, (_, i) => {
      const u = Math.min(0.995, start + i * (0.985 - start) / (segs - 1));
      const p = CURVE.getPointAt(u);
      const q = new THREE.Quaternion().setFromUnitVectors(xAxis, CURVE.getTangentAt(u));
      const sacral = i < 5;
      const r = sacral ? 0.4 - i * 0.055 : 0.11 - (i - 5) * 0.025;
      return { p, q, r, len: sacral ? 0.34 : 0.12, wide: sacral ? 1.55 - i * 0.08 : 1 };
    });
  }, [levels]);

  useFrame((state, delta) => {
    if (!animate) return;
    if (roll.current) roll.current.rotation.x += delta * 0.32;
    if (float.current) float.current.position.y = Math.sin(state.clock.elapsedTime * 0.8) * 0.06;
  });

  return (
    <group ref={float} rotation={[0.18, -0.32, 0.04]}>
      <group ref={roll} rotation={[0.35, 0, 0]}>
        <group position={[0, -0.25, 0]}>
          {placed.map(({ lv, p, q }, i) => (
            <group key={i} position={p} quaternion={q}>
              <Vertebra level={lv} bone={bone} />
            </group>
          ))}
          {discs.map((d, i) => (
            <group key={`d${i}`} position={d.p} quaternion={d.q}>
              <mesh material={disc} rotation={[0, 0, Math.PI / 2]}>
                <cylinderGeometry args={[d.r * 0.95, d.r * 0.95, d.len, 24]} />
              </mesh>
            </group>
          ))}
          {/* Sacrum: five fused, tapering segments, then a short coccyx */}
          {sacrum.map((sg, i) => (
            <group key={`s${i}`} position={sg.p} quaternion={sg.q}>
              <mesh material={bone} rotation={[0, 0, Math.PI / 2]} scale={[1, 1, sg.wide]}>
                <cylinderGeometry args={[sg.r, sg.r * 1.04, sg.len, 20]} />
              </mesh>
              {i < 4 && (
                <mesh material={bone} position={[0, sg.r * 0.95, 0]} scale={[1, 0.6, 1]}>
                  <sphereGeometry args={[sg.r * 0.38, 12, 10]} />
                </mesh>
              )}
            </group>
          ))}
        </group>
      </group>
    </group>
  );
}

export default function Spine3D({ className }: { className?: string }) {
  const wrap = useRef<HTMLDivElement>(null);
  const [visible, setVisible] = useState(true);
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    setReduced(window.matchMedia("(prefers-reduced-motion: reduce)").matches);
    const el = wrap.current;
    if (!el) return;
    const io = new IntersectionObserver(([e]) => setVisible(e.isIntersecting), { threshold: 0.05 });
    io.observe(el);
    return () => io.disconnect();
  }, []);

  return (
    <div ref={wrap} className={className} aria-label="Rotating 3D model of the human spine" role="img">
      <Canvas
        frameloop={visible && !reduced ? "always" : "demand"}
        dpr={[1, 2]}
        camera={{ position: [0, 0.45, 8.3], fov: 32 }}
        gl={{ antialias: true, alpha: true, powerPreference: "low-power" }}
      >
        <ambientLight intensity={0.55} color="#fff3e2" />
        <hemisphereLight args={["#fff6e8", "#1f5c8f", 0.45]} />
        <directionalLight position={[-4, 6, 6]} intensity={1.6} color="#fff1dc" />
        <directionalLight position={[5, -2, -6]} intensity={0.9} color="#8fbde0" />
        <pointLight position={[0, 3, 4]} intensity={0.6} color="#ffe2b8" />
        <Spine animate={!reduced} />
      </Canvas>
    </div>
  );
}

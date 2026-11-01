import { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { RoomEnvironment } from 'three/addons/environments/RoomEnvironment.js';
import type { MapUrls } from '../api';
import type { MaterialEntry } from '../../shared/catalog';
export default function CubePreview({ maps, entry }: { maps: MapUrls; entry: MaterialEntry }) {
  const host = useRef<HTMLDivElement>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    const node = host.current;
    if (!node) return;
    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: 'low-power' });
    } catch {
      setFailed(true);
      return;
    }
    setFailed(false);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 0);
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 0.95;
    node.appendChild(renderer.domElement);
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(34, 1, 0.1, 100);
    camera.position.set(3.4, 2.6, 4.2);
    const pmrem = new THREE.PMREMGenerator(renderer);
    const room = new RoomEnvironment();
    const environment = pmrem.fromScene(room, 0.04);
    scene.environment = environment.texture;
    room.dispose();
    scene.add(new THREE.AmbientLight(0xffffff, 0.45));
    const sun = new THREE.DirectionalLight(0xfff3df, 2.2);
    sun.position.set(3, 5, 4);
    scene.add(sun);
    const fill = new THREE.DirectionalLight(0xcce3d8, 0.6);
    fill.position.set(-3, 0, -1);
    scene.add(fill);
    const material = new THREE.MeshStandardMaterial({
      roughness: 0.8,
      metalness: entry.f0 >= 230 ? 0.9 : 0,
      envMapIntensity: 0.65,
    });
    const geometry = new THREE.BoxGeometry(1.85, 1.85, 1.85);
    const cube = new THREE.Mesh(geometry, material);
    cube.rotation.y = -0.1;
    scene.add(cube);
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enablePan = false;
    controls.enableZoom = false;
    controls.enableDamping = true;
    controls.minPolarAngle = 0.25;
    controls.maxPolarAngle = Math.PI - 0.25;
    controls.autoRotate = !window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    controls.autoRotateSpeed = 0.45;
    let disposed = false;
    const textures: THREE.Texture[] = [];
    const loader = new THREE.TextureLoader();
    const load = (url: string) =>
      new Promise<THREE.Texture>((resolve, reject) =>
        loader.load(
          url,
          (t) => {
            if (disposed) {
              t.dispose();
              reject(new Error('disposed'));
              return;
            }
            textures.push(t);
            resolve(t);
          },
          undefined,
          reject,
        ),
      );
    Promise.all([load(maps.albedo), load(maps.normal), load(maps.specular)])
      .then(([albedo, normal, specular]) => {
        albedo.colorSpace = THREE.SRGBColorSpace;
        albedo.magFilter = THREE.NearestFilter;
        normal.magFilter = THREE.NearestFilter;
        material.map = albedo;
        material.normalMap = normal;
        material.normalMapType = THREE.TangentSpaceNormalMap;
        material.normalScale.set(0.6, -0.6);
        // The browser preview decodes LabPBR smoothness into Three's roughness channel.
        const image = specular.image as HTMLImageElement;
        const canvas = document.createElement('canvas');
        canvas.width = image.width;
        canvas.height = image.height;
        const ctx = canvas.getContext('2d');
        if (ctx) {
          ctx.drawImage(image, 0, 0);
          const data = ctx.getImageData(0, 0, canvas.width, canvas.height);
          for (let i = 0; i < data.data.length; i += 4) {
            const value = Math.round((1 - data.data[i] / 255) ** 2 * 255);
            data.data[i] = value;
            data.data[i + 1] = value;
            data.data[i + 2] = value;
            data.data[i + 3] = 255;
          }
          ctx.putImageData(data, 0, 0);
          const rough = new THREE.CanvasTexture(canvas);
          textures.push(rough);
          material.roughnessMap = rough;
          material.roughness = 1;
        }
        material.transparent = entry.recipe === 'glass' || entry.recipe === 'leaves';
        material.alphaTest = 0.05;
        material.needsUpdate = true;
      })
      .catch(() => {
        if (!disposed) setFailed(true);
      });
    const resize = () => {
      const width = node.clientWidth;
      const height = node.clientHeight;
      if (!width || !height) return;
      renderer.setSize(width, height);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
    };
    const observer = new ResizeObserver(resize);
    observer.observe(node);
    resize();
    renderer.setAnimationLoop(() => {
      controls.update();
      renderer.render(scene, camera);
    });
    return () => {
      disposed = true;
      observer.disconnect();
      renderer.setAnimationLoop(null);
      controls.dispose();
      geometry.dispose();
      material.dispose();
      textures.forEach((t) => t.dispose());
      environment.dispose();
      pmrem.dispose();
      renderer.dispose();
      renderer.domElement.remove();
    };
  }, [maps.albedo, maps.normal, maps.specular, entry.f0, entry.recipe]);
  return (
    <div className="cube-stage">
      <div
        className="cube-canvas"
        ref={host}
        role="img"
        aria-label={`Interactive 3D preview of ${entry.label}. Drag to rotate.`}
      />
      {failed && <img className="cube-fallback" src={maps.albedo} alt={entry.label} />}
      <span className="cube-hint">
        {failed ? 'Texture preview' : 'Drag to rotate · illustrative lighting'}
      </span>
      <div className="cube-shadow" />
    </div>
  );
}

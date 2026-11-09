// 刊本的電子地圖：讀 `<名>.json`，在底圖上疊舊圖、箭頭、區域、地名。
// 用法：地圖/?圖=南徼圖 。數據格式見 docs/地圖.md。

const OFM = "https://tiles.openfreemap.org";
const DEM = "https://tiles.mapterhorn.com/tilejson.json";
const CJK = '"Noto Serif CJK TC", "Source Han Serif TC", "Noto Serif CJK SC", "Songti TC", serif';
const INK = "#3b2a1a";

function fail(msg) {
  const e = document.getElementById("err");
  e.textContent = msg;
  e.style.display = "grid";
}

// 古：紙色底、自然地球暈渲、水系，不畫今日的路、界、地名，好疊歷史內容。
function ancientStyle() {
  return {
    version: 8,
    glyphs: `${OFM}/fonts/{fontstack}/{range}.pbf`,
    sources: {
      ne: {
        type: "raster",
        tiles: [`${OFM}/natural_earth/ne2sr/{z}/{x}/{y}.png`],
        tileSize: 256,
        maxzoom: 6,
        attribution: "Natural Earth",
      },
      omt: { type: "vector", url: `${OFM}/planet` },
    },
    layers: [
      { id: "background", type: "background", paint: { "background-color": "#ece4cf" } },
      {
        id: "ne",
        type: "raster",
        source: "ne",
        paint: { "raster-opacity": 0.55, "raster-saturation": -0.6, "raster-contrast": -0.1 },
      },
      {
        id: "water",
        type: "fill",
        source: "omt",
        "source-layer": "water",
        paint: { "fill-color": "#b9c7c0" },
      },
      {
        id: "river",
        type: "line",
        source: "omt",
        "source-layer": "waterway",
        filter: ["==", ["get", "class"], "river"],
        paint: {
          "line-color": "#9fb3ad",
          "line-width": ["interpolate", ["linear"], ["zoom"], 4, 0.6, 10, 2],
        },
      },
    ],
  };
}

const BASES = {
  古: ancientStyle,
  淺: () => `${OFM}/styles/positron`,
  街: () => `${OFM}/styles/liberty`,
};

// 由經緯度算墨卡托下的方位角（度，正北為 0，順時針）。
function bearing(a, b) {
  const y = (d) => Math.log(Math.tan(Math.PI / 4 + (d * Math.PI) / 360));
  const dx = ((b[0] - a[0]) * Math.PI) / 180;
  const dy = y(b[1]) - y(a[1]);
  return (Math.atan2(dx, dy) * 180) / Math.PI;
}

// 經過各點的平滑曲綫（Catmull-Rom）。
function smooth(pts, n = 12) {
  if (pts.length < 3) return pts;
  const out = [];
  for (let i = 0; i < pts.length - 1; i++) {
    const p0 = pts[Math.max(i - 1, 0)], p1 = pts[i], p2 = pts[i + 1], p3 = pts[Math.min(i + 2, pts.length - 1)];
    for (let k = 0; k < n; k++) {
      const t = k / n, t2 = t * t, t3 = t2 * t;
      out.push([0, 1].map((j) =>
        0.5 * (2 * p1[j] + (-p0[j] + p2[j]) * t + (2 * p0[j] - 5 * p1[j] + 4 * p2[j] - p3[j]) * t2 +
          (-p0[j] + 3 * p1[j] - 3 * p2[j] + p3[j]) * t3)));
    }
  }
  out.push(pts[pts.length - 1]);
  return out;
}

// 箭頭圖標（SDF，顏色由圖層給）。
function arrowIcon() {
  const s = 48, c = document.createElement("canvas");
  c.width = c.height = s;
  const g = c.getContext("2d");
  g.fillStyle = "#fff";
  g.beginPath();
  g.moveTo(s / 2, 4);
  g.lineTo(s - 8, s - 6);
  g.lineTo(s / 2, s - 16);
  g.lineTo(8, s - 6);
  g.closePath();
  g.fill();
  return g.getImageData(0, 0, s, s);
}

// 戰地圖標：圈中一個叉（SDF）。
function battleIcon() {
  const s = 32, c = document.createElement("canvas");
  c.width = c.height = s;
  const g = c.getContext("2d");
  g.strokeStyle = "#fff";
  g.lineWidth = 3;
  g.beginPath();
  g.arc(s / 2, s / 2, s / 2 - 4, 0, 2 * Math.PI);
  g.moveTo(10, 10);
  g.lineTo(s - 10, s - 10);
  g.moveTo(s - 10, 10);
  g.lineTo(10, s - 10);
  g.stroke();
  return g.getImageData(0, 0, s, s);
}

function feature(geometry, props) {
  return { type: "Feature", geometry, properties: props };
}

// 把 json 裏的 features 換成 GeoJSON：綫、箭頭尖、區、地名。
function collect(spec) {
  const lines = [], heads = [], areas = [], places = [];
  for (const f of spec.features ?? []) {
    const color = f.color ?? "#b03a2e";
    const label = f.label ?? "";
    if (f.type === "arrow" || f.type === "line") {
      const path = f.curve ? smooth(f.path) : f.path;
      lines.push(feature({ type: "LineString", coordinates: path }, {
        color, label, width: f.width ?? 3, dash: f.dash ? 1 : 0,
      }));
      if (f.type === "arrow" && path.length > 1) {
        const a = path[path.length - 2], b = path[path.length - 1];
        heads.push(feature({ type: "Point", coordinates: b }, {
          color, rot: bearing(a, b), size: (f.width ?? 3) / 3,
        }));
      }
    } else if (f.type === "area") {
      const ring = [...f.ring];
      if (String(ring[0]) !== String(ring[ring.length - 1])) ring.push(ring[0]);
      areas.push(feature({ type: "Polygon", coordinates: [ring] }, {
        color, label, opacity: f.opacity ?? 0.25,
      }));
    } else if (f.type === "place") {
      places.push(feature({ type: "Point", coordinates: f.at }, {
        color: f.color ?? INK, label, symbol: f.symbol ?? "city",
      }));
    }
  }
  const fc = (features) => ({ type: "FeatureCollection", features });
  return { lines: fc(lines), heads: fc(heads), areas: fc(areas), places: fc(places) };
}

function vertical(id, source, extra = {}) {
  return {
    id,
    type: "symbol",
    source,
    layout: {
      "text-field": ["get", "label"],
      "text-font": ["Noto Sans Regular"],
      "text-writing-mode": ["vertical"],
      "text-size": 15,
      "text-letter-spacing": 0.1,
      ...extra.layout,
    },
    paint: {
      "text-color": ["get", "color"],
      "text-halo-color": "rgba(248,244,232,0.9)",
      "text-halo-width": 1.5,
      ...extra.paint,
    },
    ...(extra.filter ? { filter: extra.filter } : {}),
  };
}

function addOverlays(map, spec) {
  const d = collect(spec);
  (spec.rasters ?? []).forEach((r, i) => {
    map.addSource(`raster${i}`, { type: "image", url: r.url, coordinates: r.corners });
    map.addLayer({
      id: `raster${i}`,
      type: "raster",
      source: `raster${i}`,
      paint: { "raster-opacity": r.opacity ?? 0.75, "raster-fade-duration": 0 },
    });
  });
  // 現成的 GeoJSON（如 CHGIS 摘錄）：面畫成淡色區、虛綫邊，點畫小圓點；都照 label 竪排寫名。
  // 每項可寫網址，或 {url, color}。
  (spec.geojson ?? []).forEach((g, i) => {
    const { url, color = "#8a6d1f" } = typeof g === "string" ? { url: g } : g;
    const c = ["coalesce", ["get", "color"], color];
    const id = `gj${i}`;
    const poly = ["any", ["==", ["geometry-type"], "Polygon"], ["==", ["geometry-type"], "MultiPolygon"]];
    // 郡名寫在 kind 為 pref-label 的點上；沒有這種點的數據就寫在面上。
    const isLabel = ["==", ["get", "kind"], "pref-label"];
    const point = ["all", ["==", ["geometry-type"], "Point"], ["!", isLabel]];
    map.addSource(id, { type: "geojson", data: url });
    map.addLayer({
      id: `${id}-fill`, type: "fill", source: id, filter: poly,
      paint: { "fill-color": c, "fill-opacity": ["coalesce", ["get", "opacity"], 0.12] },
    });
    map.addLayer({
      id: `${id}-line`, type: "line", source: id, filter: ["!=", ["geometry-type"], "Point"],
      paint: { "line-color": c, "line-width": ["coalesce", ["get", "width"], 1.2], "line-dasharray": [4, 2] },
    });
    map.addLayer({
      id: `${id}-pt`, type: "circle", source: id, filter: point,
      paint: { "circle-radius": 2.2, "circle-color": c },
    });
    map.addLayer({
      id: `${id}-pt-label`, type: "symbol", source: id, filter: point, minzoom: 6.5,
      layout: {
        "text-field": ["get", "label"], "text-font": ["Noto Sans Regular"], "text-size": 11,
        "text-writing-mode": ["vertical"], "text-anchor": "left", "text-offset": [0.5, 0],
      },
      paint: { "text-color": c, "text-halo-color": "rgba(248,244,232,0.9)", "text-halo-width": 1 },
    });
    map.addLayer({
      id: `${id}-label`, type: "symbol", source: id, filter: ["any", isLabel, poly],
      layout: {
        "text-field": ["get", "label"], "text-font": ["Noto Sans Regular"], "text-size": 16,
        "text-writing-mode": ["vertical"], "text-letter-spacing": 0.4,
      },
      paint: { "text-color": c, "text-opacity": 0.85, "text-halo-color": "rgba(248,244,232,0.8)", "text-halo-width": 1 },
    });
  });
  for (const k of ["areas", "lines", "heads", "places"]) map.addSource(k, { type: "geojson", data: d[k] });
  map.addLayer({
    id: "areas", type: "fill", source: "areas",
    paint: { "fill-color": ["get", "color"], "fill-opacity": ["get", "opacity"] },
  });
  map.addLayer({
    id: "areas-edge", type: "line", source: "areas",
    paint: { "line-color": ["get", "color"], "line-width": 1.2, "line-dasharray": [3, 2] },
  });
  map.addLayer(vertical("areas-label", "areas", { layout: { "text-size": 18, "text-letter-spacing": 0.4 }, paint: { "text-opacity": 0.85 } }));
  const line = (id, dash) => ({
    id, type: "line", source: "lines",
    filter: ["==", ["get", "dash"], dash ? 1 : 0],
    layout: { "line-cap": "round", "line-join": "round" },
    paint: {
      "line-color": ["get", "color"],
      "line-width": ["get", "width"],
      ...(dash ? { "line-dasharray": [2, 1.5] } : {}),
    },
  });
  map.addLayer(line("lines", false));
  map.addLayer(line("lines-dash", true));
  map.addImage("arrow", arrowIcon(), { sdf: true });
  map.addLayer({
    id: "heads", type: "symbol", source: "heads",
    layout: {
      "icon-image": "arrow",
      "icon-size": ["*", 0.45, ["get", "size"]],
      "icon-rotate": ["get", "rot"],
      "icon-rotation-alignment": "map",
      "icon-allow-overlap": true,
      "icon-ignore-placement": true,
    },
    paint: { "icon-color": ["get", "color"] },
  });
  map.addLayer({
    id: "lines-label", type: "symbol", source: "lines",
    layout: {
      "symbol-placement": "line-center",
      "text-field": ["get", "label"],
      "text-font": ["Noto Sans Regular"],
      "text-size": 14,
      "text-offset": [0, -1],
    },
    paint: { "text-color": ["get", "color"], "text-halo-color": "rgba(248,244,232,0.9)", "text-halo-width": 1.5 },
  });
  map.addLayer({
    id: "places", type: "circle", source: "places",
    paint: {
      "circle-radius": ["match", ["get", "symbol"], "capital", 5, "battle", 0, 3.5],
      "circle-color": ["match", ["get", "symbol"], "capital", "#fff", ["get", "color"]],
      "circle-stroke-color": ["get", "color"],
      "circle-stroke-width": ["match", ["get", "symbol"], "capital", 2, 0],
    },
  });
  map.addImage("battle", battleIcon(), { sdf: true });
  map.addLayer({
    id: "battle", type: "symbol", source: "places",
    filter: ["==", ["get", "symbol"], "battle"],
    layout: { "icon-image": "battle", "icon-size": 0.55, "icon-allow-overlap": true },
    paint: { "icon-color": ["get", "color"] },
  });
  map.addLayer(vertical("places-label", "places", {
    layout: { "text-anchor": "left", "text-offset": [0.7, 0] },
  }));
}

// 舊圖透明度的滑桿。
function panel(map, spec) {
  const p = document.getElementById("panel");
  (spec.rasters ?? []).forEach((r, i) => {
    const l = document.createElement("label");
    l.textContent = (r.label ?? "舊圖") + " ";
    const s = document.createElement("input");
    Object.assign(s, { type: "range", min: 0, max: 1, step: 0.05, value: r.opacity ?? 0.75 });
    s.oninput = () => map.setPaintProperty(`raster${i}`, "raster-opacity", Number(s.value));
    l.append(s);
    p.append(l);
  });
}

async function main() {
  const q = new URLSearchParams(location.search);
  const name = q.get("圖") ?? q.get("map");
  if (!name) return fail("未指定地圖（地圖/?圖=名）");
  let spec;
  try {
    const r = await fetch(`${encodeURIComponent(name)}.json`);
    if (!r.ok) throw new Error(r.status);
    spec = await r.json();
  } catch {
    return fail(`找不到地圖「${name}」`);
  }
  document.title = spec.title ?? name;
  // 截靜態替補圖時不要按鈕。
  if (q.has("still")) document.body.classList.add("still");
  const base = (BASES[spec.base] ?? BASES.古)();
  const map = new maplibregl.Map({
    container: "map",
    style: base,
    center: spec.center ?? [112.45, 34.67],
    zoom: spec.zoom ?? 5,
    // 給了 bounds 就讓這片地方正好填滿框（插圖框的長寬隨版式而變）。
    ...(spec.bounds ? { bounds: spec.bounds, fitBoundsOptions: { padding: 10 } } : {}),
    pitch: spec.pitch ?? 0,
    bearing: spec.bearing ?? 0,
    maxPitch: 80,
    localIdeographFontFamily: CJK,
    attributionControl: { compact: true, customAttribution: spec.source ?? "" },
    canvasContextAttributes: { preserveDrawingBuffer: q.has("still") },
  });
  map.addControl(new maplibregl.NavigationControl({ visualizePitch: true }));
  map.addControl(new maplibregl.FullscreenControl());
  // 版權說明默認收起，點 ⓘ 展開。
  if (!q.has("still"))
    map.once("load", () =>
      document.querySelector(".maplibregl-ctrl-attrib")?.classList.remove("maplibregl-compact-show"));
  map.on("style.load", () => {
    if (spec.globe !== false) map.setProjection({ type: "globe" });
    if (spec.terrain !== false) {
      map.addSource("dem", { type: "raster-dem", url: DEM, tileSize: 512 });
      map.addSource("dem-shade", { type: "raster-dem", url: DEM, tileSize: 512 });
      map.addLayer({
        id: "hillshade", type: "hillshade", source: "dem-shade",
        paint: { "hillshade-shadow-color": "#6b5a40", "hillshade-exaggeration": 0.35 },
      });
      map.addControl(new maplibregl.TerrainControl({ source: "dem", exaggeration: 1.5 }));
      if (spec.pitch) map.setTerrain({ source: "dem", exaggeration: 1.5 });
    }
    addOverlays(map, spec);
    panel(map, spec);
  });
  // 截靜態圖用：地圖畫完後在 body 上標記。
  map.on("idle", () => document.body.setAttribute("data-idle", "1"));
}

main();

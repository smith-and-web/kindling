// Serialises the live kindling document into SVG primitives (rects, text, icon paths).
// Runs inside the webview; returns markup plus the characters used per font face so
// the caller can embed subset fonts. Geometry comes from layout, not from pixels.
// Words inside [data-demo-type] are tagged so the recorder can animate typing.
window.__dom2svg = (prefix = "s") => {
  const W = innerWidth;
  const H = innerHeight;
  const defs = [];
  const chars = {};
  let currentLayers = [];
  const typed = [];
  let seq = 0;
  const uid = (k) => `${prefix}${k}${seq++}`;
  const SVG_NS = "http://www.w3.org/2000/svg";
  // Elements inside iframes must be read with their own window's computed styles.
  const gcs = (el, pseudo) => el.ownerDocument.defaultView.getComputedStyle(el, pseudo);
  const n = (v) => Math.round(v * 10) / 10;
  const esc = (s) =>
    s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);

  // Resolve any CSS colour (including oklch/color-mix) to sRGB via a 1px canvas.
  const cv = document.createElement("canvas");
  cv.width = cv.height = 1;
  const cx = cv.getContext("2d", { willReadFrequently: true });
  const colorCache = new Map();
  const color = (c) => {
    if (!c || c === "transparent" || c === "none") return null;
    if (colorCache.has(c)) return colorCache.get(c);
    let out = null;
    const m = c.match(/^rgba?\(([^)]+)\)$/);
    if (m) {
      const p = m[1]
        .split(/[\s,/]+/)
        .filter(Boolean)
        .map(Number);
      const a = p.length > 3 ? p[3] : 1;
      if (a > 0.003) out = { rgb: `rgb(${p[0]},${p[1]},${p[2]})`, a };
    } else {
      cx.clearRect(0, 0, 1, 1);
      cx.fillStyle = "#010203";
      cx.fillStyle = c;
      if (cx.fillStyle === "#010203") {
        colorCache.set(c, null);
        return null;
      }
      cx.fillRect(0, 0, 1, 1);
      const d = cx.getImageData(0, 0, 1, 1).data;
      if (d[3] > 0) out = { rgb: `rgb(${d[0]},${d[1]},${d[2]})`, a: d[3] / 255 };
    }
    colorCache.set(c, out);
    return out;
  };
  const paint = (attr, c) =>
    c ? ` ${attr}="${c.rgb}"${c.a < 1 ? ` ${attr}-opacity="${n(c.a)}"` : ""}` : "";

  const metricCache = new Map();
  const ascent = (cs) => {
    const font = `${cs.fontStyle} ${cs.fontWeight} ${cs.fontSize} ${cs.fontFamily}`;
    if (!metricCache.has(font)) {
      cx.font = font;
      const m = cx.measureText("Hg");
      metricCache.set(font, { asc: m.fontBoundingBoxAscent, desc: m.fontBoundingBoxDescent });
    }
    return metricCache.get(font);
  };
  const EMBEDDED = new Set(["Inter", "Fraunces", "Newsreader"]);
  const faceOf = (cs) => {
    const list = cs.fontFamily.split(",").map((f) => f.trim().replace(/^["']|["']$/g, ""));
    const fam = list.find((f) => EMBEDDED.has(f)) || list[list.length - 1];
    return { fam, key: `${fam}|${cs.fontStyle === "italic" ? "italic" : "normal"}` };
  };
  const addChars = (cs, text) => {
    const { fam, key } = faceOf(cs);
    if (!EMBEDDED.has(fam)) return;
    chars[key] = (chars[key] || "") + text;
  };
  const fontAttrs = (cs) => {
    const { fam } = faceOf(cs);
    let s = `font-family:${EMBEDDED.has(fam) ? `'${fam}'` : fam};font-size:${cs.fontSize};font-weight:${cs.fontWeight}`;
    if (cs.fontStyle !== "normal") s += `;font-style:${cs.fontStyle}`;
    if (cs.letterSpacing !== "normal" && cs.letterSpacing !== "0px")
      s += `;letter-spacing:${cs.letterSpacing}`;
    if (cs.fontVariationSettings !== "normal")
      s += `;font-variation-settings:${cs.fontVariationSettings.replace(/"/g, "'")}`;
    if (cs.fontFeatureSettings !== "normal")
      s += `;font-feature-settings:${cs.fontFeatureSettings.replace(/"/g, "'")}`;
    if (cs.fontVariantNumeric !== "normal") s += `;font-variant-numeric:${cs.fontVariantNumeric}`;
    return s;
  };
  const transformText = (t, cs) =>
    cs.textTransform === "uppercase"
      ? t.toUpperCase()
      : cs.textTransform === "lowercase"
        ? t.toLowerCase()
        : cs.textTransform === "capitalize"
          ? t.replace(/\b\p{L}/gu, (c) => c.toUpperCase())
          : t;

  const radius = (cs, r) => {
    const raw = cs.borderTopLeftRadius;
    const v = raw.endsWith("%") ? (parseFloat(raw) / 100) * r.width : parseFloat(raw) || 0;
    return Math.min(v, r.width / 2, r.height / 2);
  };
  const shape = (r, rx, extra) =>
    `<rect x="${n(r.left)}" y="${n(r.top)}" width="${n(r.width)}" height="${n(r.height)}"${rx ? ` rx="${n(rx)}"` : ""}${extra}/>`;

  const shadows = (cs, r, rx, out, wantInset = false) => {
    if (cs.boxShadow === "none") return;
    const parts = cs.boxShadow.split(/,(?![^(]*\))/);
    for (const part of parts.reverse()) {
      const inset = /inset/.test(part);
      if (inset !== wantInset) continue;
      const col = part.match(/(rgba?\([^)]+\)|#[0-9a-f]+|oklch\([^)]+\)|color\([^)]+\))/i);
      const nums = part
        .replace(col ? col[0] : "", "")
        .trim()
        .split(/\s+/)
        .map(parseFloat)
        .filter((v) => !Number.isNaN(v));
      const [ox = 0, oy = 0, blur = 0, spread = 0] = nums;
      const c = color(col ? col[0] : "rgba(0,0,0,0.2)");
      if (!c) continue;
      if (inset) {
        if (blur > 0) continue;
        if (spread > 0)
          out.push(
            shape(
              {
                left: r.left + spread / 2,
                top: r.top + spread / 2,
                width: r.width - spread,
                height: r.height - spread,
              },
              Math.max(0, rx - spread / 2),
              ` fill="none"${paint("stroke", c)} stroke-width="${n(spread)}"`
            )
          );
        if (oy)
          out.push(
            shape(
              {
                left: r.left,
                top: oy > 0 ? r.top : r.bottom + oy,
                width: r.width,
                height: Math.abs(oy),
              },
              0,
              paint("fill", c)
            )
          );
        if (ox)
          out.push(
            shape(
              {
                left: ox > 0 ? r.left : r.right + ox,
                top: r.top,
                width: Math.abs(ox),
                height: r.height,
              },
              0,
              paint("fill", c)
            )
          );
        continue;
      }
      const box = {
        left: r.left + ox - spread,
        top: r.top + oy - spread,
        width: r.width + spread * 2,
        height: r.height + spread * 2,
      };
      let f = "";
      if (blur > 0) {
        const id = uid("f");
        defs.push(
          `<filter id="${id}" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="${n(blur / 2)}"/></filter>`
        );
        f = ` filter="url(#${id})"`;
      }
      out.push(shape(box, rx + spread, `${paint("fill", c)}${f}`));
    }
  };

  const borders = (cs, r, rx, out) => {
    const side = (s) => ({
      w: parseFloat(cs[`border${s}Width`]) || 0,
      c: cs[`border${s}Style`] === "none" ? null : color(cs[`border${s}Color`]),
    });
    const t = side("Top"),
      rt = side("Right"),
      b = side("Bottom"),
      l = side("Left");
    const same = [rt, b, l].every((s) => s.w === t.w && s.c?.rgb === t.c?.rgb && s.c?.a === t.c?.a);
    if (same && t.w > 0 && t.c) {
      const inset = {
        left: r.left + t.w / 2,
        top: r.top + t.w / 2,
        width: r.width - t.w,
        height: r.height - t.w,
      };
      out.push(
        shape(
          inset,
          Math.max(0, rx - t.w / 2),
          ` fill="none"${paint("stroke", t.c)} stroke-width="${t.w}"`
        )
      );
      return;
    }
    const line = (s, x, y, w, h) => {
      if (s.w > 0 && s.c)
        out.push(shape({ left: x, top: y, width: w, height: h }, 0, paint("fill", s.c)));
    };
    line(t, r.left, r.top, r.width, t.w);
    line(b, r.left, r.bottom - b.w, r.width, b.w);
    line(l, r.left, r.top, l.w, r.height);
    line(rt, r.right - rt.w, r.top, rt.w, r.height);
  };

  const inlineSvgImage = (href) => {
    try {
      const x = new XMLHttpRequest();
      x.open("GET", href, false);
      x.send();
      if (x.status >= 200 && x.status < 300)
        return `data:image/svg+xml;base64,${btoa(unescape(encodeURIComponent(x.responseText)))}`;
    } catch {
      /* fall through */
    }
    return null;
  };
  const imageCache = new Map();
  const imageHref = (src) => {
    if (imageCache.has(src)) return imageCache.get(src);
    let href = null;
    if (/^data:/.test(src)) href = src;
    else if (/\.svg(\?|$)/.test(src)) href = inlineSvgImage(src);
    imageCache.set(src, href);
    return href;
  };

  // Top-level comma split that ignores commas inside parentheses.
  const splitTop = (v) => {
    const out = [];
    let depth = 0,
      cur = "";
    for (const ch of v) {
      if (ch === "(") depth++;
      if (ch === ")") depth--;
      if (ch === "," && depth === 0) {
        out.push(cur.trim());
        cur = "";
      } else cur += ch;
    }
    if (cur.trim()) out.push(cur.trim());
    return out;
  };
  // Evaluate "12px", "40%", "calc(50% - 0.75px)" against a reference length.
  const len = (v, ref) => {
    v = v.trim();
    const m = v.match(/^calc\((.*)\)$/);
    const expr = m ? m[1] : v;
    let total = 0;
    for (const t of expr.replace(/\s([+-])\s/g, " $1").split(/\s+/)) {
      const sign = t.startsWith("-") ? -1 : 1;
      const body = t.replace(/^[+-]/, "");
      if (body.endsWith("%")) total += (sign * parseFloat(body) * ref) / 100;
      else total += sign * (parseFloat(body) || 0);
    }
    return total;
  };
  const backgrounds = (cs, r, rx, out) => {
    const images = splitTop(cs.backgroundImage);
    const positions = splitTop(cs.backgroundPosition);
    const sizes = splitTop(cs.backgroundSize);
    const bl = parseFloat(cs.borderLeftWidth) || 0,
      bt = parseFloat(cs.borderTopWidth) || 0;
    const box = {
      left: r.left + bl,
      top: r.top + bt,
      width: r.width - bl - (parseFloat(cs.borderRightWidth) || 0),
      height: r.height - bt - (parseFloat(cs.borderBottomWidth) || 0),
    };
    // CSS paints the first layer on top.
    for (let i = images.length - 1; i >= 0; i--) {
      const img = images[i];
      const size = (sizes[i] || sizes[0] || "auto").split(/\s+(?![^(]*\))/);
      const pos = (positions[i] || positions[0] || "0% 0%").split(/\s+(?![^(]*\))/);
      const tw =
        size[0] === "auto" || size[0] === "cover" || size[0] === "contain"
          ? box.width
          : len(size[0], box.width);
      const th =
        !size[1] || size[1] === "auto"
          ? size[0] === "auto" || /cover|contain/.test(size[0])
            ? box.height
            : tw
          : len(size[1], box.height);
      const tx = box.left + len(pos[0] || "0%", box.width - tw);
      const ty = box.top + len(pos[1] || "50%", box.height - th);
      const tile = { left: tx, top: ty, width: tw, height: th };
      const u = img.match(/^url\("?([^")]+)"?\)/);
      if (u) {
        const href = imageHref(u[1]);
        if (href)
          out.push(
            `<image href="${esc(href)}" x="${n(tx)}" y="${n(ty)}" width="${n(tw)}" height="${n(th)}" preserveAspectRatio="xMidYMid meet"/>`
          );
        continue;
      }
      const g = img.match(/^(repeating-)?linear-gradient\((.*)\)$/s);
      if (!g) continue;
      const args = splitTop(g[2]);
      let angle = 180;
      if (/^-?[\d.]+(deg|turn|rad)$/.test(args[0])) {
        const a = args.shift();
        angle = a.endsWith("turn")
          ? parseFloat(a) * 360
          : a.endsWith("rad")
            ? (parseFloat(a) * 180) / Math.PI
            : parseFloat(a);
      } else if (/^to /.test(args[0])) {
        const a = args.shift();
        const map = {
          "to top": 0,
          "to right": 90,
          "to bottom": 180,
          "to left": 270,
          "to top right": 45,
          "to right top": 45,
          "to bottom right": 135,
          "to right bottom": 135,
          "to bottom left": 225,
          "to left bottom": 225,
          "to top left": 315,
          "to left top": 315,
        };
        angle = map[a] ?? 180;
      }
      const rad = (angle * Math.PI) / 180;
      const dx = Math.sin(rad),
        dy = -Math.cos(rad);
      const L = Math.abs(tw * dx) + Math.abs(th * dy);
      const cxm = tx + tw / 2,
        cym = ty + th / 2;
      const stops = [];
      args.forEach((a, k) => {
        const cm = a.match(/^(rgba?\([^)]+\)|#[0-9a-f]+|[a-z]+\([^)]*\)|[a-z]+)\s*(.*)$/i);
        if (!cm) return;
        const c = color(cm[1]) || { rgb: "rgb(0,0,0)", a: 0 };
        const offs = cm[2] ? splitTop(cm[2].replace(/\s+(?![^(]*\))/g, ",")) : [];
        if (!offs.length) offs.push(null);
        for (const o of offs) stops.push({ c, o: o == null ? null : len(o, L) / L });
      });
      stops.forEach((st, k) => {
        if (st.o == null) st.o = stops.length === 1 ? 0 : k / (stops.length - 1);
      });
      let last = 0;
      const id = uid("g");
      defs.push(
        `<linearGradient id="${id}" gradientUnits="userSpaceOnUse" x1="${n(cxm - (dx * L) / 2)}" y1="${n(cym - (dy * L) / 2)}" x2="${n(cxm + (dx * L) / 2)}" y2="${n(cym + (dy * L) / 2)}">${stops
          .map((st) => {
            last = Math.max(last, Math.min(1, Math.max(0, st.o)));
            return `<stop offset="${n(last)}" stop-color="${st.c.rgb}"${st.c.a < 1 ? ` stop-opacity="${n(st.c.a)}"` : ""}/>`;
          })
          .join("")}</linearGradient>`
      );
      out.push(shape(tile, tw === box.width && th === box.height ? rx : 0, ` fill="url(#${id})"`));
    }
  };

  const rotation = (el) => {
    let deg = 0;
    for (let e = el, i = 0; e && i < 3; e = e.parentElement, i++) {
      const t = gcs(e).transform;
      if (t && t !== "none") {
        const m = t.match(/matrix\(([^)]+)\)/);
        if (m) {
          const [a, b] = m[1].split(",").map(Number);
          deg += (Math.atan2(b, a) * 180) / Math.PI;
        }
      }
    }
    return Math.round(deg);
  };

  const icon = (el, r, out) => {
    const cs = gcs(el);
    const clone = el.cloneNode(true);
    const src = [el, ...el.querySelectorAll("*")];
    const dst = [clone, ...clone.querySelectorAll("*")];
    src.forEach((s, i) => {
      const d = dst[i];
      const c = gcs(s);
      d.removeAttribute("class");
      d.removeAttribute("style");
      for (const a of ["data-testid", "aria-hidden", "focusable", "role"]) d.removeAttribute(a);
      if (i === 0) return;
      const fill = color(c.fill);
      const stroke = c.stroke === "none" ? null : color(c.stroke);
      d.setAttribute("fill", fill ? fill.rgb : "none");
      if (fill && fill.a < 1) d.setAttribute("fill-opacity", n(fill.a));
      if (stroke) {
        d.setAttribute("stroke", stroke.rgb);
        if (stroke.a < 1) d.setAttribute("stroke-opacity", n(stroke.a));
        d.setAttribute("stroke-width", c.strokeWidth);
        d.setAttribute("stroke-linecap", c.strokeLinecap);
        d.setAttribute("stroke-linejoin", c.strokeLinejoin);
      } else d.removeAttribute("stroke");
      if (c.opacity !== "1") d.setAttribute("opacity", c.opacity);
    });
    const w = parseFloat(cs.width) || r.width;
    const h = parseFloat(cs.height) || r.height;
    if (!clone.getAttribute("viewBox")) clone.setAttribute("viewBox", `0 0 ${w} ${h}`);
    const cxm = r.left + r.width / 2,
      cym = r.top + r.height / 2;
    clone.setAttribute("x", n(cxm - w / 2));
    clone.setAttribute("y", n(cym - h / 2));
    clone.setAttribute("width", n(w));
    clone.setAttribute("height", n(h));
    clone.setAttribute("overflow", "visible");
    // Root fill/stroke defaults for children relying on inheritance.
    const rootStroke = cs.stroke === "none" ? null : color(cs.stroke);
    const rootFill = color(cs.fill);
    clone.setAttribute("fill", rootFill ? rootFill.rgb : "none");
    if (rootStroke) clone.setAttribute("stroke", rootStroke.rgb);
    const deg = rotation(el);
    const html = clone.outerHTML.replace(/ xmlns="[^"]*"/, "");
    out.push(deg ? `<g transform="rotate(${deg} ${n(cxm)} ${n(cym)})">${html}</g>` : html);
  };

  // Text nodes: one <text> per line with an absolutely positioned tspan per word.
  const ranges = new Map();
  const widest = (rects) => [...rects].reduce((a, q) => (!a || q.width > a.width ? q : a), null);
  const textNode = (node, cs, out, ellipsisEl, clampEl) => {
    const data = node.data;
    if (!ranges.has(node.ownerDocument))
      ranges.set(node.ownerDocument, node.ownerDocument.createRange());
    const range = ranges.get(node.ownerDocument);
    if (!data.trim()) return;
    const col = color(cs.color);
    if (!col) return;
    const { asc } = ascent(cs);
    const lines = [];
    const re = /\S+/g;
    let m;
    while ((m = re.exec(data))) {
      range.setStart(node, m.index);
      range.setEnd(node, m.index + m[0].length);
      const rects = [...range.getClientRects()].filter((q) => q.width > 0);
      if (!rects.length) continue;
      if (rects.length === 1) {
        lines.push({ x: rects[0].left, top: rects[0].top, right: rects[0].right, t: m[0] });
        continue;
      }
      // Word broken across lines (or ligature boxes): place characters individually.
      for (let i = 0; i < m[0].length; i++) {
        range.setStart(node, m.index + i);
        range.setEnd(node, m.index + i + 1);
        const q = widest(range.getClientRects());
        if (q && q.width > 0) lines.push({ x: q.left, top: q.top, right: q.right, t: m[0][i] });
      }
    }
    let clipId = null;
    let ell = null;
    if (clampEl) {
      // -webkit-line-clamp: WebKit lays out the hidden lines; end the last shown one with "…".
      const cr = clampEl.getBoundingClientRect();
      const ccs = gcs(clampEl);
      const bottom =
        cr.bottom - (parseFloat(ccs.paddingBottom) || 0) - (parseFloat(ccs.borderBottomWidth) || 0);
      const shown = lines.filter((w) => w.top + 1 < bottom);
      if (shown.length < lines.length && shown.length) {
        const last = shown[shown.length - 1];
        ell = { x: last.right, top: last.top };
      }
      lines.splice(0, lines.length, ...shown);
    }
    if (!lines.length) return;
    if (ellipsisEl) {
      // WebKit reports rects only for the glyphs it kept; draw "…" after the last one.
      const er = ellipsisEl.getBoundingClientRect();
      const ecs = gcs(ellipsisEl);
      const contentRight =
        er.right - (parseFloat(ecs.paddingRight) || 0) - (parseFloat(ecs.borderRightWidth) || 0);
      cx.font = `${cs.fontStyle} ${cs.fontWeight} ${cs.fontSize} ${cs.fontFamily}`;
      const ew = cx.measureText("…").width;
      let cut = er.left;
      for (let i = 0; i < data.length; i++) {
        if (/\s/.test(data[i])) continue;
        range.setStart(node, i);
        range.setEnd(node, i + 1);
        const q = widest(range.getClientRects());
        if (!q || q.width === 0 || q.right > contentRight - ew + 0.75) break;
        cut = q.right;
      }
      clipId = uid("e");
      defs.push(
        `<clipPath id="${clipId}"><rect x="${n(er.left - 50)}" y="${n(er.top - 50)}" width="${n(cut - er.left + 50)}" height="${n(er.height + 100)}"/></clipPath>`
      );
      ell = { x: cut, top: lines[0].top };
    }
    const byLine = new Map();
    for (const w of lines) {
      const k = Math.round(w.top);
      if (!byLine.has(k)) byLine.set(k, []);
      byLine.get(k).push(w);
    }
    const style = fontAttrs(cs);
    const parts = [];
    const typing = !!node.parentElement?.closest("[data-demo-type]");
    const { desc } = ascent(cs);
    for (const [, words] of byLine) {
      const y = n(words[0].top + asc);
      const spans = words
        .map((w) => {
          const t = transformText(w.t, cs);
          addChars(cs, t);
          if (!typing) return `<tspan x="${n(w.x)}">${esc(t)}</tspan>`;
          typed.push({ x: w.x, right: w.right, top: w.top, height: asc + desc });
          return `<tspan x="${n(w.x)}" data-i="${typed.length - 1}">${esc(t)}</tspan>`;
        })
        .join("");
      parts.push(`<text y="${y}">${spans}</text>`);
      const deco = cs.textDecorationLine;
      if (deco && deco !== "none") {
        const x0 = words[0].x,
          x1 = words[words.length - 1].right;
        const dc = color(cs.textDecorationColor) || col;
        const th = Math.max(1, parseFloat(cs.fontSize) / 16);
        if (deco.includes("underline"))
          parts.push(
            `<rect x="${n(x0)}" y="${n(words[0].top + asc + th * 1.5)}" width="${n(x1 - x0)}" height="${n(th)}"${paint("fill", dc)}/>`
          );
        if (deco.includes("line-through"))
          parts.push(
            `<rect x="${n(x0)}" y="${n(words[0].top + asc * 0.68)}" width="${n(x1 - x0)}" height="${n(th)}"${paint("fill", dc)}/>`
          );
      }
    }
    out.push(
      `<g style="${style}"${paint("fill", col)}${clipId ? ` clip-path="url(#${clipId})"` : ""}>${parts.join("")}</g>`
    );
    if (ell) {
      addChars(cs, "…");
      out.push(
        `<text x="${n(ell.x)}" y="${n(ell.top + asc)}" style="${style}"${paint("fill", col)}>…</text>`
      );
    }
  };

  const fieldText = (el, cs, r, out) => {
    let text = el.value;
    let c = color(cs.color);
    if (!text && el.placeholder) {
      text = el.placeholder;
      const pcs = gcs(el, "::placeholder");
      c = color(pcs.color) || c;
    }
    if (el.tagName === "SELECT") text = el.options[el.selectedIndex]?.text || "";
    if (!text || !c) return;
    const { asc, desc } = ascent(cs);
    const pl = (parseFloat(cs.paddingLeft) || 0) + (parseFloat(cs.borderLeftWidth) || 0);
    const pt = (parseFloat(cs.paddingTop) || 0) + (parseFloat(cs.borderTopWidth) || 0);
    const pb = (parseFloat(cs.paddingBottom) || 0) + (parseFloat(cs.borderBottomWidth) || 0);
    const style = fontAttrs(cs);
    const lines = el.tagName === "TEXTAREA" ? text.split("\n") : [text];
    const lh = parseFloat(cs.lineHeight) || (asc + desc) * 1.2;
    const clip = uid("i");
    defs.push(`<clipPath id="${clip}">${shape(r, 0, "")}</clipPath>`);
    const body = lines
      .map((line, i) => {
        const y =
          el.tagName === "TEXTAREA"
            ? r.top + pt + i * lh + (lh - asc - desc) / 2 + asc
            : r.top + pt + (r.height - pt - pb - asc - desc) / 2 + asc;
        addChars(cs, line);
        return `<text x="${n(r.left + pl - (el.scrollLeft || 0))}" y="${n(y)}" xml:space="preserve">${esc(line)}</text>`;
      })
      .join("");
    out.push(`<g style="${style}"${paint("fill", c)} clip-path="url(#${clip})">${body}</g>`);
  };

  const intersects = (r, clip) =>
    r.right > clip.left && r.left < clip.right && r.bottom > clip.top && r.top < clip.bottom;

  const visit = (el, out, clip, deferred, inheritedClamp = null) => {
    const cs = gcs(el);
    if (cs.display === "none") return;
    if (el.id?.startsWith("qa-") || el.hasAttribute?.("data-demo-skip")) return;
    const z = parseInt(cs.zIndex, 10);
    const positioned = cs.position !== "static";
    const inTop = el.matches("dialog:modal, :popover-open");
    if (
      !deferred &&
      (inTop || (positioned && (cs.position === "fixed" || z > 0))) &&
      el !== el.ownerDocument.body
    ) {
      // The top layer (modal dialogs, open popovers) paints above every z-index.
      const top = (el.tagName === "DIALOG" && el.matches(":modal")) || el.matches(":popover-open");
      currentLayers.push({
        el,
        z: top ? 1e9 + currentLayers.length : Number.isNaN(z) ? 0 : z,
        order: currentLayers.length,
      });
      return;
    }
    const opacity = parseFloat(cs.opacity);
    if (opacity === 0) return;
    if (cs.clipPath === "inset(50%)" || /rect\(0px,? 0px,? 0px,? 0px\)/.test(cs.clip)) return;
    const r = el.getBoundingClientRect();
    const visible = cs.visibility !== "hidden";
    const clipsSelf = cs.overflowX !== "visible" || cs.overflowY !== "visible";
    if (clipsSelf && !intersects(r, clip)) return;
    const own = [];
    if (el.tagName === "DIALOG" && el.matches(":modal")) {
      const bd = color(gcs(el, "::backdrop").backgroundColor);
      if (bd) out.push(`<rect width="${W}" height="${H}"${paint("fill", bd)}/>`);
    }
    if (visible && r.width > 0 && r.height > 0 && intersects(r, clip)) {
      const rx = radius(cs, r);
      shadows(cs, r, rx, own);
      const bg = color(cs.backgroundColor);
      if (bg) own.push(shape(r, rx, paint("fill", bg)));
      shadows(cs, r, rx, own, true);
      if (cs.backgroundImage && cs.backgroundImage !== "none") backgrounds(cs, r, rx, own);
      borders(cs, r, rx, own);
      if (el.tagName === "svg" && el.namespaceURI === SVG_NS) {
        icon(el, r, own);
      } else if (el.tagName === "IMG" && el.currentSrc) {
        const href = imageHref(el.currentSrc);
        if (href)
          own.push(
            `<image href="${esc(href)}" x="${n(r.left)}" y="${n(r.top)}" width="${n(r.width)}" height="${n(r.height)}"/>`
          );
      } else if (
        /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName) &&
        !/^(checkbox|radio|range|hidden)$/.test(el.type)
      ) {
        fieldText(el, cs, r, own);
      }
      if (cs.outlineStyle !== "none" && parseFloat(cs.outlineWidth) > 0) {
        const ow = parseFloat(cs.outlineWidth),
          off = parseFloat(cs.outlineOffset) || 0;
        const c = color(cs.outlineColor);
        const o = {
          left: r.left - off - ow / 2,
          top: r.top - off - ow / 2,
          width: r.width + (off + ow / 2) * 2,
          height: r.height + (off + ow / 2) * 2,
        };
        if (c)
          own.push(
            shape(o, rx ? rx + off : 0, ` fill="none"${paint("stroke", c)} stroke-width="${ow}"`)
          );
      }
    }
    if (el.namespaceURI === SVG_NS) {
      if (own.length)
        out.push(opacity < 1 ? `<g opacity="${opacity}">${own.join("")}</g>` : own.join(""));
      return;
    }
    const kids = [];
    let childClip = clip;
    if (clipsSelf) {
      childClip = {
        left: Math.max(clip.left, r.left),
        top: Math.max(clip.top, r.top),
        right: Math.min(clip.right, r.right),
        bottom: Math.min(clip.bottom, r.bottom),
      };
    }
    const ellipsis =
      cs.textOverflow === "ellipsis" && el.scrollWidth > el.clientWidth + 1 ? el : null;
    const clamp =
      cs.webkitLineClamp && cs.webkitLineClamp !== "none" && el.scrollHeight > el.clientHeight + 1
        ? el
        : inheritedClamp;
    if (el.tagName === "IFRAME") {
      kids.push(frameContent(el, cs, r));
    }
    // A closed <details> renders only its summary.
    const closed = el.tagName === "DETAILS" && !el.open;
    for (const child of el.childNodes) {
      if (child.nodeType === 3) {
        if (visible && !closed) textNode(child, cs, kids, ellipsis, clamp);
      } else if (child.nodeType === 1 && (!closed || child.tagName === "SUMMARY"))
        visit(child, kids, childClip, false, clamp);
    }
    let body = own.join("");
    if (kids.length) {
      if (clipsSelf) {
        const id = uid("c");
        defs.push(`<clipPath id="${id}">${shape(r, radius(cs, r), "")}</clipPath>`);
        body += `<g clip-path="url(#${id})">${kids.join("")}</g>`;
      } else body += kids.join("");
    }
    if (!body) return;
    out.push(opacity < 1 ? `<g opacity="${opacity}">${body}</g>` : body);
  };

  // Positioned layers paint above normal flow, in z-index then document order.
  const paintLayers = (list, out, view) => {
    for (let i = 0; i < list.length; i++) {
      const sorted = list.slice(i).sort((a, b) => a.z - b.z || a.order - b.order);
      list.splice(i, list.length - i, ...sorted);
      const tmp = [];
      visit(list[i].el, tmp, view, true);
      out.push(tmp.join(""));
    }
  };
  const paintDocument = (doc, out, view) => {
    const saved = currentLayers;
    currentLayers = [];
    const bg =
      color(gcs(doc.body).backgroundColor) || color(gcs(doc.documentElement).backgroundColor);
    if (bg)
      out.push(
        `<rect x="${view.left}" y="${view.top}" width="${view.right - view.left}" height="${view.bottom - view.top}"${paint("fill", bg)}/>`
      );
    visit(doc.body, out, view, false);
    paintLayers(currentLayers, out, view);
    currentLayers = saved;
  };
  // Iframes (the export preview) are painted in their own coordinates. A sandboxed
  // srcdoc frame is read through the same-origin twin made by __dom2svgPrepare.
  const frameContent = (frame, cs, r) => {
    const doc = frame.contentDocument || window.__dom2svgTwins?.get(frame)?.contentDocument;
    if (!doc?.body) return "";
    const ox = r.left + (parseFloat(cs.borderLeftWidth) || 0) + (parseFloat(cs.paddingLeft) || 0);
    const oy = r.top + (parseFloat(cs.borderTopWidth) || 0) + (parseFloat(cs.paddingTop) || 0);
    const view = { left: 0, top: 0, right: frame.clientWidth, bottom: frame.clientHeight };
    const inner = [];
    paintDocument(doc, inner, view);
    const id = uid("c");
    defs.push(
      `<clipPath id="${id}"><rect width="${view.right}" height="${view.bottom}"/></clipPath>`
    );
    return `<g transform="translate(${n(ox)} ${n(oy)})" clip-path="url(#${id})">${inner.join("")}</g>`;
  };

  const out = [];
  paintDocument(document, out, { left: 0, top: 0, right: W, bottom: H });
  return { width: W, height: H, body: out.join(""), defs: defs.join(""), chars, typed };
};

// Sandboxed srcdoc iframes hide their document from the parent. Load the same markup
// into an invisible twin of identical size (scripts stay disabled) so it can be read.
window.__dom2svgPrepare = () => {
  window.__dom2svgCleanup();
  const twins = (window.__dom2svgTwins = new Map());
  for (const frame of document.querySelectorAll("iframe")) {
    if (frame.contentDocument || !frame.srcdoc) continue;
    const twin = document.createElement("iframe");
    twin.setAttribute("sandbox", "allow-same-origin");
    twin.setAttribute("data-demo-skip", "");
    twin.setAttribute("aria-hidden", "true");
    twin.style.cssText = `position:fixed;left:-${frame.clientWidth + 100}px;top:0;border:0;width:${frame.clientWidth}px;height:${frame.clientHeight}px;visibility:hidden`;
    twin.srcdoc = frame.srcdoc;
    document.body.append(twin);
    twins.set(frame, twin);
  }
  return twins.size;
};
window.__dom2svgReady = () =>
  [...(window.__dom2svgTwins?.values() || [])].every(
    (t) => t.contentDocument?.readyState === "complete" && t.contentDocument.body
  );
window.__dom2svgCleanup = () => {
  for (const twin of window.__dom2svgTwins?.values() || []) twin.remove();
  window.__dom2svgTwins = new Map();
};

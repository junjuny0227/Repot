// Light and dark captures of the app; the browser picks by color scheme and density.
export function Screenshot({ name, alt, sizes, eager = false }: { name: string; alt: string; sizes: string; eager?: boolean }) {
  const base = `${import.meta.env.BASE_URL}screenshots/${name}`;
  const set = (mode: string) => `${base}-${mode}-1100.webp 1100w, ${base}-${mode}.webp 2200w`;
  return (
    <picture className="screenshot">
      <source media="(prefers-color-scheme: dark)" srcSet={set("dark")} sizes={sizes} />
      <img
        src={`${base}-light-1100.webp`}
        srcSet={set("light")}
        sizes={sizes}
        width={1100}
        height={720}
        alt={alt}
        loading={eager ? "eager" : "lazy"}
        decoding="async"
        fetchPriority={eager ? "high" : "auto"}
      />
    </picture>
  );
}

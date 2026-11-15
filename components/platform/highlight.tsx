import { ENTITY_LABELS, type Entity, type EntityType } from "@/lib/nlp";

export function HighlightedText({ text, entities, hidden = [] }: { text: string; entities: Entity[]; hidden?: EntityType[] }) {
  const parts: React.ReactNode[] = [];
  let last = 0;
  entities.forEach((e, i) => {
    if (hidden.includes(e.type)) return;
    if (e.start < last) return;
    parts.push(text.slice(last, e.start));
    parts.push(
      <mark
        key={i}
        className={`ent ent-${e.type} ${e.negated ? "ent-negated" : ""} bg-transparent text-inherit`}
        title={`${ENTITY_LABELS[e.type]}${e.negated ? " (negated)" : ""}${e.normalized ? ` · ${e.normalized}` : ""}`}
      >
        {text.slice(e.start, e.end)}
      </mark>,
    );
    last = e.end;
  });
  parts.push(text.slice(last));
  return <p className="whitespace-pre-wrap">{parts}</p>;
}

export const ENTITY_SWATCH: Record<EntityType, string> = {
  symptom: "ent-symptom",
  anatomy: "ent-anatomy",
  duration: "ent-duration",
  medication: "ent-medication",
  therapy: "ent-therapy",
  imaging: "ent-imaging",
  redflag: "ent-redflag",
  procedure: "ent-procedure",
};

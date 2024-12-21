const keywords = [
  "as",
  "do",
  "if",
  "in",
  "of",

  "for",
  "new",
  "try",
  "var",

  "async",
  "await",
  "break",
  "case",
  "catch",
  "class",
  "const",
  "continue",
  "debugger",
  "default",
  "delete",
  "else",
  "enum",
  "export",
  "extends",
  "false",
  "finally",
  "function",
  "import",
  "instanceof",
  "null",
  "return",
  "super",
  "switch",
  "this",
  "throw",
  "true",
  "typeof",
  "void",
  "while",
  "with",
  "yield",
];
console.log("num words:", keywords.length);

function fmtNum(num: number): string {
  let pow = 0;
  if (num < 1_000_000) return num.toString();

  while (num >= 1_000 && pow < 4) {
    num /= 1_000;
    pow += 1;
  }

  return `${num.toFixed(1)}${["", "K", "M", "B", "T"][pow]}`;
}

function sum(a: number[]): number {
  let s = 0;
  for (const n of a) s += n;
  return s;
}

function sumSq(a: number[]): number {
  let s = 0;
  for (const n of a) {
    if (Array.isArray(n)) continue;

    const k = n * n;
    s += k * k;
  }
  return s;
}

function code(word: string, index: number) {
  if (word.length <= index) return 0;
  return word.charCodeAt(index);
}

function* permutations(values: readonly number[]): Generator<number[]> {
  if (values.length <= 1) {
    yield [...values];
    return;
  }

  for (let i = 0; i < values.length; i++) {
    const value = values[i];
    const subPermInput = [...values.slice(0, i), ...values.slice(i + 1)];

    for (const perm of permutations(subPermInput)) {
      perm.push(value);
      yield perm;
    }
  }
}

class MaxCounter<T = unknown> {
  maxVal: number = 0;
  readonly records: T[] = [];

  constructor() {}

  record(value: number, info: T) {
    if (value > this.maxVal) {
      this.maxVal = value;
      this.records.length = 0;
    }

    if (value === this.maxVal) {
      this.records.push(info);
    }
  }
}

class IntRange {
  constructor(
    readonly low: number,
    readonly high: number,
  ) {}

  get size(): number {
    return Math.floor(this.high - this.low);
  }

  *[Symbol.iterator]() {
    for (let i = this.low; i < this.high; i++) {
      yield i;
    }
  }
}

type Selections = readonly Iterable<unknown>[];
type ExtractSelections<T extends Selections> = {
  [K in keyof T]: T[K] extends Iterable<infer U> ? U : never;
};

function* selections<T extends Selections>(
  ...selections: T
): Generator<ExtractSelections<T>> {
  function* selectionsInternal(
    selections: readonly Iterable<unknown>[],
    index: number,
  ): Generator<unknown[]> {
    if (index < 0) {
      yield [];
      return;
    }

    for (const parentVal of selectionsInternal(selections, index - 1)) {
      for (const value of selections[index]) {
        yield [...parentVal, value];
      }
    }
  }

  yield* selectionsInternal(selections, selections.length - 1) as any;
}

function runHash() {}

// -----------------------------------------------------------------------------
//
//                            TEST CASES
//
// -----------------------------------------------------------------------------

function javaHash(base: number, slots: number[]) {
  // Gets to 40 w/ 128 slots (base 2, slots [2, 1, 0, 3])
  const lowKey = "a".charCodeAt(0);
  return function hash(word: string) {
    return (
      (code(word, slots[0]) - lowKey) * base * base * base +
      (code(word, slots[1]) - lowKey) * base * base +
      (code(word, slots[2]) - lowKey) * base +
      (code(word, slots[3]) - lowKey)
    );
  };
}

function testHash1() {
  const counter = new MaxCounter();

  const perms = [...permutations([0, 1, 2, 3])];
  const slotCounts = [64, 128, 256];
  for (const [i, slotCount, perm] of selections(
    new IntRange(0, 1000),
    slotCounts,
    perms,
  )) {
    const hash = javaHash(i, perm);
    const values = new Set(keywords.map(hash).map((h) => h % slotCount));
    counter.record(values.size, [i, slotCount, perm]);
  }

  console.log(
    counter.maxVal,
    counter.records.sort((a: any, b: any) => a[1] - b[1]),
  );
}

function testHash2() {
  function baseHash(
    slots: number[],
    base0: number,
    base1: number,
    base2: number,
    base3: number,
  ) {
    const lowKey = "a".charCodeAt(0);
    return function hash(word: string) {
      const data = [
        code(word, 0) - lowKey,
        code(word, 1) - lowKey,
        code(word, 2) - lowKey,
        code(word, 3) - lowKey,
        code(word, word.length - 1) - lowKey,
        // word.length,
      ];

      return (
        data[slots[0]] * base0 * base0 * base0 * base2 * base2 +
        data[slots[1]] * base0 * base0 * base2 * base1 +
        data[slots[2]] * base0 * base1 * base1 * base3 +
        data[slots[3]] * base1 * base1 * base1 * base3 * base3 +
        data[slots[4]] * 0 +
        0
      );
    };
  }

  const counter = new MaxCounter();

  const baseRange = new IntRange(0, 64);
  const perms = [...permutations([0, 1, 2, 3, 4])];
  const slotCounts = [64];

  let iter = 0;
  const estTotalIter =
    perms.length *
    baseRange.size *
    baseRange.size *
    baseRange.size *
    baseRange.size;

  for (const [slotCount, perm, base0, base1, base2, base3] of selections(
    slotCounts,
    perms,
    baseRange,
    baseRange,
    baseRange,
    baseRange,
  )) {
    const hash = baseHash(perm, base0, base1, base2, base3);
    const values = new Set(keywords.map(hash).map((h) => h % slotCount));
    counter.record(values.size, [slotCount, base0, base1, base2, base3]);

    if (++iter % (1024 * 1024) === 0) {
      console.log(`iter ${fmtNum(iter)} / ${fmtNum(estTotalIter)}`);
    }
  }

  console.log(
    counter.maxVal,
    counter.records
      .sort(
        (a: any, b: any) => a[0] * 1000 + sumSq(a) - (b[0] * 1000 + sumSq(b)),
      )
      .slice(0, 10),
  );
}

function printSolutions() {
  const firstHash = javaHash(2, [2, 1, 0, 3]);
  const map = new Map(keywords.map((k) => [(firstHash(k) % 128 + 128) % 128, k]));
  console.log("java1", map);
}

// testHash1();
testHash2();
// printSolutions();

// for (const perm of selections(new IntRange(0, 5), [1, 2, 3])) {
//   console.log(perm);
// }

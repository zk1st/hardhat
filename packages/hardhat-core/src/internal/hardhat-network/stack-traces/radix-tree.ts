export class RadixNode {
  public isPresent: boolean;
  public charsMatchedBefore: number;
  public edgeLabel: string;
  public childNodes: Map<string, RadixNode>;

  constructor(
    edgeLabel: string,
    isPresent: boolean,
    charsMatchedBefore: number
  ) {
    this.childNodes = new Map<string, RadixNode>();
    this.edgeLabel = edgeLabel;
    this.isPresent = isPresent;
    this.charsMatchedBefore = charsMatchedBefore;
  }
}

export class RadixTree {
  public root = new RadixNode("", false, 0);

  public add(word: string) {
    let currentNode = this.root;

    for (let i = 0; i < word.length; i++) {
      const char = word[i];

      const nextNode = currentNode.childNodes.get(char);
      if (nextNode === undefined) {
        const node = new RadixNode(
          word.substring(i),
          true,
          currentNode.charsMatchedBefore + currentNode.edgeLabel.length
        );

        currentNode.childNodes.set(char, node);
        return;
      }

      // We know it's at least 1
      const prefixLength = getSharedPrefixLength(word, i, nextNode.edgeLabel);

      // The next node's label is included in the word
      if (prefixLength === nextNode.edgeLabel.length) {
        // The next node matches the word exactly
        if (prefixLength + i === word.length) {
          nextNode.isPresent = true;
          return;
        }

        i += prefixLength - 1;
        currentNode = nextNode;
        continue;
      }

      // If the edgeLabel includes what's left of the word and some extra
      if (prefixLength + i === word.length) {
        // nextNode includes the current word and some extra, so we insert a
        // new node with the word
        const node = new RadixNode(
          word.substring(i),
          true,
          currentNode.charsMatchedBefore + currentNode.edgeLabel.length
        );

        // The current node now points to the new node
        currentNode.childNodes.set(char, node);

        // The new node points to nextNode
        node.childNodes.set(nextNode.edgeLabel[prefixLength], nextNode);

        // nextNode edgeLabel and charsMatchedBefore get updated
        nextNode.edgeLabel = nextNode.edgeLabel.substring(prefixLength);
        nextNode.charsMatchedBefore =
          node.charsMatchedBefore + node.edgeLabel.length;

        return;
      }

      // The edgeLabel includes some part of what's left, but not all of it
      // insert a new inbetween node between current node and it's child, that
      // will have children for the old child and a new node for the given word.
      const middleNode = new RadixNode(
        word.substring(i, i + prefixLength),
        false,
        currentNode.charsMatchedBefore + currentNode.edgeLabel.length
      );

      // nextNode should come after middleNode and its label and charsMatchedBefore need to be adapted
      middleNode.childNodes.set(nextNode.edgeLabel[prefixLength], nextNode);
      nextNode.edgeLabel = nextNode.edgeLabel.substring(prefixLength);
      nextNode.charsMatchedBefore =
        middleNode.charsMatchedBefore + middleNode.edgeLabel.length;

      // Set the middleNode as currenNode's child
      currentNode.childNodes.set(char, middleNode);

      // Create a new node for the word
      const newNode = new RadixNode(
        word.substring(i + prefixLength),
        true,
        middleNode.charsMatchedBefore + middleNode.edgeLabel.length
      );
      middleNode.childNodes.set(word[i + prefixLength], newNode);

      return;
    }
  }

  public getMaxMatch(
    word: string,
    firstNode = this.root
  ): [boolean, number, RadixNode] {
    let currentNode = firstNode;

    while (true) {
      const prefixLength = getSharedPrefixLength(
        word,
        currentNode.charsMatchedBefore,
        currentNode.edgeLabel
      );

      const matched = prefixLength + currentNode.charsMatchedBefore;

      const entireWordMatched = matched === word.length;
      const entireEdgeLabelMatched =
        prefixLength === currentNode.edgeLabel.length;

      if (!entireWordMatched) {
        // Node label not completely matched, the word is not present
        if (!entireEdgeLabelMatched) {
          // console.log("CASE 0: label nor word consumed");
          return [false, matched, currentNode];
        }

        const nextNode = currentNode.childNodes.get(word[matched]);

        // No next node found, the word is not present
        if (nextNode === undefined) {
          // console.log("CASE 1: can't continue");
          return [false, matched, currentNode];
        }

        // console.log("CASE 2: continue");
        currentNode = nextNode;
        continue;
      }

      // The word gets exactly matched in this node, so we may have found the
      // word
      if (entireEdgeLabelMatched) {
        // console.log("CASE 3: entire match");
        return [currentNode.isPresent, matched, currentNode];
      }

      // console.log("CASE 4: label not consumed");
      return [false, matched, currentNode];
    }
  }

  /**
   * Returns a list of words found in the tree after node.
   */
  public *getDescendantSuffixes(node: RadixNode): Iterable<string> {
    if (node.isPresent) {
      yield node.edgeLabel;
    }

    for (const child of node.childNodes.values()) {
      for (const childDescendent of this.getDescendantSuffixes(child)) {
        yield node.edgeLabel + childDescendent;
      }
    }
  }
}

function getSharedPrefixLength(a: string, aOffset: number, b: string): number {
  const maxIndex = Math.min(a.length - aOffset, b.length);

  let i: number;
  for (i = 0; i < maxIndex; i++) {
    if (a[i + aOffset] !== b[i]) {
      break;
    }
  }

  return i;
}

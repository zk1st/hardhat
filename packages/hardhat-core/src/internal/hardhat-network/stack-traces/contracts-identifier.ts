import { createNonCryptographicHashBasedIdentifier } from "../../util/hash";
import {
  normalizeLibraryRuntimeBytecodeIfNecessary,
  zeroOutAddresses,
  zeroOutSlices,
} from "./library-utils";
import { EvmMessageTrace, isCreateTrace } from "./message-trace";
import { Bytecode } from "./model";
import { getOpcodeLength, Opcode } from "./opcodes";
import { RadixNode, RadixTree } from "./radix-tree";

export class ContractsIdentifier {
  private _cache: Map<string, Bytecode> = new Map();

  private _bytecodes: Map<string, Bytecode> = new Map();
  private _radixTree = new RadixTree();

  constructor(private readonly _enableCache = true) {}

  public addBytecode(bytecode: Bytecode) {
    // this._trie.add(bytecode);
    const word = this._getRadixTrieWord(bytecode.normalizedCode);
    this._bytecodes.set(word, bytecode);
    this._radixTree.add(word);
    this._cache.clear();
  }

  public getBytecodeFromMessageTrace(
    trace: EvmMessageTrace
  ): Bytecode | undefined {
    const normalizedCode = normalizeLibraryRuntimeBytecodeIfNecessary(
      trace.code
    );

    let cacheKey: string | undefined;
    if (this._enableCache) {
      cacheKey = this._getCacheKey(normalizedCode);
      const cached = this._cache.get(cacheKey);

      if (cached !== undefined) {
        return cached;
      }
    }

    // const result = this._searchBytecode(trace, normalizedCode);
    const result = this._searchBytecodeOnRadixTree(trace, normalizedCode);

    if (this._enableCache) {
      if (result !== undefined) {
        const deploymentBytecodeWithArguments =
          result.isDeployment &&
          result.normalizedCode.length < normalizedCode.length;

        if (!deploymentBytecodeWithArguments) {
          this._cache.set(cacheKey!, result);
        }
      }
    }

    return result;
  }

  private _getCacheKey(normalizedCode: Buffer): string {
    return createNonCryptographicHashBasedIdentifier(normalizedCode).toString(
      "hex"
    );
  }

  private _getRadixTrieWord(normalizedCode: Buffer): string {
    return normalizedCode.toString("hex");
  }

  private _searchBytecodeOnRadixTree(
    trace: EvmMessageTrace,
    code: Buffer,
    normalizeLibraries = true,
    radixNode: RadixNode = this._radixTree.root
  ): Bytecode | undefined {
    const word = this._getRadixTrieWord(code);
    const [found, matchedChars, node] = this._radixTree.getMaxMatch(
      word,
      radixNode
    );

    if (found) {
      return this._bytecodes.get(word);
    }

    // The entire string is present as a prefix, but not exactly
    if (word.length === matchedChars) {
      return undefined;
    }

    // Deployment messages have their abi-encoded arguments at the end of the bytecode.
    //
    // We don't know how long those arguments are, as we don't know which contract is being
    // deployed, hence we don't know the signature of its constructor.
    //
    // To make things even harder, we can't trust that the user actually passed the right
    // amount of arguments.
    //
    // Luckily, the chances of a complete deployment bytecode being the prefix of another one are
    // remote. For example, most of the time it ends with its metadata hash, which will differ.
    //
    // We take advantage of this last observation, and just return the bytecode that exactly
    // matched the searchResult (sub)trie that we got.
    const entireNodeMatched =
      matchedChars === node.charsMatchedBefore + node.edgeLabel.length;
    const notEntireBytecodeFound = matchedChars < word.length;
    if (
      isCreateTrace(trace) &&
      entireNodeMatched &&
      notEntireBytecodeFound &&
      node.isPresent
    ) {
      const bytecode = this._bytecodes.get(
        word.substring(0, node.charsMatchedBefore) + node.edgeLabel
      )!;

      if (bytecode.isDeployment) {
        return bytecode;
      }
    }

    if (normalizeLibraries) {
      for (const suffix of this._radixTree.getDescendantSuffixes(node)) {
        const descendant = word.substring(0, node.charsMatchedBefore) + suffix;
        const bytecodeWithLibraries = this._bytecodes.get(descendant)!;
        if (
          bytecodeWithLibraries.libraryAddressPositions.length === 0 &&
          bytecodeWithLibraries.immutableReferences.length === 0
        ) {
          continue;
        }

        const normalizedLibrariesCode = zeroOutAddresses(
          code,
          bytecodeWithLibraries.libraryAddressPositions
        );

        const normalizedCode = zeroOutSlices(
          normalizedLibrariesCode,
          bytecodeWithLibraries.immutableReferences
        );

        const normalizedResult = this._searchBytecodeOnRadixTree(
          trace,
          normalizedCode,
          false,
          node
        );

        if (normalizedResult !== undefined) {
          return normalizedResult;
        }
      }
    }

    // If we got here we may still have the contract, but with a different metadata hash.
    //
    // We check if we got to match the entire executable bytecode, and are just stuck because
    // of the metadata. If that's the case, we can assume that any descendant will be a valid
    // Bytecode, so we just choose the most recently added one.
    //
    // The reason this works is that there's no chance that Solidity includes an entire
    // bytecode (i.e. with metadata), as a prefix of another one.
    if (this._isMatchingMetadata(code, matchedChars)) {
      const suffixes = Array.from(this._radixTree.getDescendantSuffixes(node));

      if (suffixes.length > 0) {
        // TODO: this should be the last one in chronological insertion order
        const descendant =
          word.substring(0, node.charsMatchedBefore) +
          suffixes[suffixes.length - 1];
        return this._bytecodes.get(descendant)!;
      }
    }

    return undefined;
  }

  /**
   * Returns true if the lastByte is placed right when the metadata starts or after it.
   */
  private _isMatchingMetadata(code: Buffer, lastByte: number): boolean {
    for (let byte = 0; byte < lastByte; ) {
      const opcode = code[byte];

      // Solidity always emits REVERT INVALID right before the metadata
      if (opcode === Opcode.REVERT && code[byte + 1] === Opcode.INVALID) {
        return true;
      }

      byte += getOpcodeLength(opcode);
    }

    return false;
  }
}

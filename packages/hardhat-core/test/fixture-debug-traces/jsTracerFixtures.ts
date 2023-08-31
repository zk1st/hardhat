interface JsTracerFixture {
  description: string;
  tracerCode: string;
  expected: any;
  only?: boolean;
}

export const jsTracerFixtures: JsTracerFixture[] = [
  {
    description: "should call the result callback",
    tracerCode: `{
      result: function(ctx, db) { return { hello: "world" }; },
      fault: function(log, db) {},
    }`,
    expected: { hello: "world" },
  },
  {
    description: "should have the log.op.isPush function",
    tracerCode: `{
      isPushOpcodes: 0,
      step: function(log, db) { if (log.op.isPush()) this.isPushOpcodes++; },
      result: function(ctx, db) { return this.isPushOpcodes; },
      fault: function(log, db) {},
    }`,
    expected: 18,
  },
  {
    description: "should have the log.op.toString function",
    tracerCode: `{
      opcodes: [],
      step: function(log, db) { this.opcodes.push(log.op.toString()); },
      result: function(ctx, db) { return this.opcodes; },
      fault: function(log, db) {},
    }`,
    // prettier-ignore
    expected: ["PUSH1", "PUSH1", "MSTORE", "CALLVALUE", "DUP1", "ISZERO", "PUSH1", "JUMPI", "JUMPDEST", "POP", "PUSH1", "CALLDATASIZE", "LT", "PUSH1", "JUMPI", "PUSH1", "CALLDATALOAD", "PUSH1", "SHR", "DUP1", "PUSH4", "EQ", "PUSH1", "JUMPI", "DUP1", "PUSH4", "EQ", "PUSH1", "JUMPI", "JUMPDEST", "PUSH1", "PUSH1", "DUP1", "CALLDATASIZE", "SUB", "PUSH1", "DUP2", "LT", "ISZERO", "PUSH1", "JUMPI", "JUMPDEST", "DUP2", "ADD", "SWAP1", "DUP1", "DUP1", "CALLDATALOAD", "SWAP1", "PUSH1", "ADD", "SWAP1", "SWAP3", "SWAP2", "SWAP1", "POP", "POP", "POP", "PUSH1", "JUMP", "JUMPDEST", "DUP1", "PUSH1", "DUP2", "SWAP1", "SSTORE", "POP", "POP", "JUMP", "JUMPDEST", "STOP"],
  },
  {
    description: "should have the log.stack.length function",
    tracerCode: `{
      stackLengths: [],
      fault: function(log, db) {},
      step: function(log, db) { this.stackLengths.push(log.stack.length()); },
      result: function(ctx, db) { return this.stackLengths; },
    }`,
    // prettier-ignore
    expected: [0, 1, 2, 0, 1, 2, 2, 3, 1, 1, 0, 1, 2, 1, 2, 0, 1, 1, 2, 1, 2, 3, 2, 3, 1, 2, 3, 2, 3, 1, 1, 2, 3, 4, 5, 4, 5, 6, 5, 5, 6, 4, 4, 5, 4, 4, 5, 6, 6, 6, 7, 6, 6, 6, 6, 6, 5, 4, 3, 4, 3, 3, 4, 5, 6, 6, 4, 3, 2, 1, 1],
  },
];

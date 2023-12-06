use super::source_map::SourceMap;

pub struct Function {
    pub name: String,
    pub source_map: SourceMap,
}

// export enum ContractFunctionType {
//   CONSTRUCTOR,
//   FUNCTION,
//   FALLBACK,
//   RECEIVE,
//   GETTER,
//   MODIFIER,
//   FREE_FUNCTION,
// }

// export enum ContractFunctionVisibility {
//   PRIVATE,
//   INTERNAL,
//   PUBLIC,
//   EXTERNAL,
// }

// export class ContractFunction {
//   constructor(
//     public readonly name: string,
//     public readonly type: ContractFunctionType,
//     public readonly location: SourceLocation,
//     public readonly contract?: Contract,
//     public readonly visibility?: ContractFunctionVisibility,
//     public readonly isPayable?: boolean,
//     public selector?: Buffer,
//     public readonly paramTypes?: any[]
//   ) {
//     if (contract !== undefined && !contract.location.contains(location)) {
//       throw new Error("Incompatible contract and function location");
//     }
//   }

//   public isValidCalldata(calldata: Buffer): boolean {
//     if (this.paramTypes === undefined) {
//       // if we don't know the param types, we just assume that the call is
// valid       return true;
//     }

//     return AbiHelpers.isValidCalldata(this.paramTypes, calldata);
//   }
// }

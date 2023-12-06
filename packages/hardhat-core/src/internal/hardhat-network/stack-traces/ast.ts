import {
  EnumDefinition,
  ErrorDefinition,
  EventDefinition,
  ImportDirective,
  PragmaDirective,
  StructDefinition,
  UserDefinedValueTypeDefinition,
  UsingForDirective,
} from "./ignored-ast";

export type SourceLocation = string;

export interface SourceUnit {
  id: number;
  src: SourceLocation;
  nodes: Array<
    | ContractDefinition
    | FunctionDefinition
    | VariableDeclaration
    // ignored:
    | EnumDefinition
    | ErrorDefinition
    | ImportDirective
    | PragmaDirective
    | StructDefinition
    | UserDefinedValueTypeDefinition
    | UsingForDirective
  >;
  nodeType: "SourceUnit";
}

export interface ContractDefinition {
  id: number;
  src: SourceLocation;
  name: string;
  contractKind: "contract" | "interface" | "library";
  linearizedBaseContracts: number[];
  nodes: Array<
    | FunctionDefinition
    | ModifierDefinition
    | VariableDeclaration
    // ignored:
    | StructDefinition
    | UserDefinedValueTypeDefinition
    | UsingForDirective
    | EnumDefinition
    | ErrorDefinition
    | EventDefinition
  >;
  nodeType: "ContractDefinition";
}

export interface FunctionDefinition {
  id: number;
  nodeType: "FunctionDefinition";
}

export interface FunctionDefinition {
  id: number;
  src: SourceLocation;
  name: string;
  functionSelector?: string;
  implemented: boolean;
  kind: "function" | "receive" | "constructor" | "fallback" | "freeFunction";
  parameters: ParameterList;
  returnParameters: ParameterList;
  stateMutability: StateMutability;
  visibility: Visibility;
  nodeType: "FunctionDefinition";
}

export type StateMutability = "payable" | "pure" | "nonpayable" | "view";

export interface ParameterList {
  id: number;
  src: SourceLocation;
  parameters: VariableDeclaration[];
  nodeType: "ParameterList";
}

export type Visibility = "external" | "public" | "internal" | "private";

export interface ModifierDefinition {
  id: number;
  src: SourceLocation;
  name: string;
  parameters: ParameterList;
  visibility: Visibility;
  nodeType: "ModifierDefinition";
}

export interface VariableDeclaration {
  id: number;
  src: SourceLocation;
  name: string;
  functionSelector?: string;
  indexed?: boolean;
  typeName?: TypeName | null;
  visibility: Visibility;
  nodeType: "VariableDeclaration";
}

export type TypeName =
  | ArrayTypeName
  | ElementaryTypeName
  | FunctionTypeName
  | Mapping
  | UserDefinedTypeName;

export interface ArrayTypeName {
  id: number;
  src: SourceLocation;
  typeDescriptions: TypeDescriptions;
  baseType: TypeName;
  nodeType: "ArrayTypeName";
}

export interface ElementaryTypeName {
  id: number;
  src: SourceLocation;
  typeDescriptions: TypeDescriptions;
  name: string;
  nodeType: "ElementaryTypeName";
}

export interface FunctionTypeName {
  id: number;
  src: SourceLocation;
  typeDescriptions: TypeDescriptions;
  parameterTypes: ParameterList;
  returnParameterTypes: ParameterList;
  stateMutability: StateMutability;
  visibility: Visibility;
  nodeType: "FunctionTypeName";
}

export interface Mapping {
  id: number;
  src: SourceLocation;
  typeDescriptions: TypeDescriptions;
  keyType: TypeName;
  valueType: TypeName;
  keyName?: string;
  keyNameLocation?: string;
  valueName?: string;
  valueNameLocation?: string;
  nodeType: "Mapping";
}

export interface UserDefinedTypeName {
  id: number;
  src: SourceLocation;
  typeDescriptions: TypeDescriptions;
  name?: string;
  referencedDeclaration: number;
  nodeType: "UserDefinedTypeName";
}

export interface TypeDescriptions {
  typeIdentifier?: string | null;
  typeString?: string | null;
}

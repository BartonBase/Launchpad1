
export const HybridLaunchErrorCode = {
  InvalidDecimals: 6000,
  RatioNotAllowed: 6001,
  ZeroCollectionSize: 6002,
  CollectionTooLargeForSupply: 6003,
  ReservedFeeAboveCap: 6004,
  ReservedCaptureFeeBelowRerollFee: 6005,
  ReservedFeeDestinationNotBurn: 6006,
  InvalidLaunchDestination: 6007,
  MathOverflow: 6008,
  PostLaunchCheckFailed: 6009,
  CollectionBelowMinimum: 6010,
  MintAccountInUse: 6011,
  SolFeeOutOfRange: 6012,
  CollectionAboveCap: 6013,
  GraduationThresholdOutOfRange: 6014,
  ReservedGraduationUnfundable: 6015,
  MintCostConstantStale: 6016,
  FeeRecipientInvalid: 6017,
  DbcAccountInvalid: 6018,
  DbcConfigRejected: 6019,
  DbcMintRejected: 6020,
  DbcCreatorMismatch: 6021,
  DbcAlreadyGraduated: 6022,
  DbcConfigNotApproved: 6023
};

export type HybridLaunchErrorName = keyof typeof HybridLaunchErrorCode;

export interface QaQuestion {
  id: string
  domain: string
  text: string
  /** Option strings from the API (mirrors QaQuestion.options: string[] in generated schema) */
  options: string[]
  /** Index of the selected predefined option, if answered */
  answeredIndex?: number
  /** Free-form "other" answer text, if answered with custom input */
  answeredText?: string
}

export interface QaRound {
  id: string
  roundNumber: number
  questions: QaQuestion[]
  /** Whether this round is the rollback point (restored-to marker) */
  isRollbackPoint?: boolean
}

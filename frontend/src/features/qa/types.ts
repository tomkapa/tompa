export interface QaOption {
  text: string
}

export interface QaQuestion {
  id: string
  domain: string
  text: string
  options: QaOption[]
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

import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { QuestionBlock } from './question-block'
import type { QaQuestion } from './types'

const question: QaQuestion = {
  id: 'q1',
  domain: 'backend',
  text: 'Which approach should we use?',
  rationale: 'This decision affects the core architecture.',
  options: [
    { label: 'Option A', pros: 'Fast to implement', cons: 'Less flexible' },
    { label: 'Option B', pros: 'Very flexible', cons: 'Slower to build' },
    { label: 'Option C', pros: 'Well-known pattern', cons: 'Verbose boilerplate' },
  ],
  recommendedIndex: 0,
}

describe('QuestionBlock', () => {
  it('renders question text', () => {
    render(
      <QuestionBlock
        question={question}
        onAnswer={vi.fn()}
        isRollbackPoint={false}
        answered={false}
      />
    )
    expect(screen.getByText('Which approach should we use?')).toBeInTheDocument()
  })

  it('renders rationale text', () => {
    render(
      <QuestionBlock
        question={question}
        onAnswer={vi.fn()}
        isRollbackPoint={false}
        answered={false}
      />
    )
    expect(screen.getByText('This decision affects the core architecture.')).toBeInTheDocument()
  })

  it('renders all option cards', () => {
    render(
      <QuestionBlock
        question={question}
        onAnswer={vi.fn()}
        isRollbackPoint={false}
        answered={false}
      />
    )
    expect(screen.getByText('Option A')).toBeInTheDocument()
    expect(screen.getByText('Option B')).toBeInTheDocument()
    expect(screen.getByText('Option C')).toBeInTheDocument()
  })

  it('selecting an option calls onAnswer with correct args', () => {
    const onAnswer = vi.fn()
    render(
      <QuestionBlock
        question={question}
        onAnswer={onAnswer}
        isRollbackPoint={false}
        answered={false}
      />
    )
    fireEvent.click(screen.getByLabelText('Select Option B'))
    expect(onAnswer).toHaveBeenCalledWith('q1', 1, null)
  })

  it('allows reselecting a different answer after answering', () => {
    const onAnswer = vi.fn()
    const answeredQuestion: QaQuestion = { ...question, answeredIndex: 0 }
    render(
      <QuestionBlock
        question={answeredQuestion}
        onAnswer={onAnswer}
        isRollbackPoint={false}
        answered={true}
      />
    )
    fireEvent.click(screen.getByLabelText('Select Option B'))
    expect(onAnswer).toHaveBeenCalledWith('q1', 1, null)
  })

  it('renders domain tag', () => {
    render(
      <QuestionBlock
        question={question}
        onAnswer={vi.fn()}
        isRollbackPoint={false}
        answered={false}
      />
    )
    expect(screen.getByText('backend')).toBeInTheDocument()
  })

  it('shows AI suggested badge on recommended option', () => {
    render(
      <QuestionBlock
        question={question}
        onAnswer={vi.fn()}
        isRollbackPoint={false}
        answered={false}
      />
    )
    expect(screen.getByText('AI suggested')).toBeInTheDocument()
  })

  it('does not show AI suggested badge after answering', () => {
    const answeredQuestion: QaQuestion = { ...question, answeredIndex: 1 }
    render(
      <QuestionBlock
        question={answeredQuestion}
        onAnswer={vi.fn()}
        isRollbackPoint={false}
        answered={true}
      />
    )
    expect(screen.queryByText('AI suggested')).not.toBeInTheDocument()
  })
})

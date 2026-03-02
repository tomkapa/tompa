import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { QuestionBlock } from './question-block'
import type { QaQuestion } from './types'

const question: QaQuestion = {
  id: 'q1',
  domain: 'backend',
  text: 'Which approach should we use?',
  options: ['Option A', 'Option B', 'Option C'],
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

  it('renders all option cards', () => {
    render(
      <QuestionBlock
        question={question}
        onAnswer={vi.fn()}
        isRollbackPoint={false}
        answered={false}
      />
    )
    expect(screen.getByRole('button', { name: 'Option A' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Option B' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Option C' })).toBeInTheDocument()
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
    fireEvent.click(screen.getByRole('button', { name: 'Option B' }))
    expect(onAnswer).toHaveBeenCalledWith('q1', 1, null)
  })

  it('does not call onAnswer when already answered', () => {
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
    fireEvent.click(screen.getByRole('button', { name: 'Option B' }))
    expect(onAnswer).not.toHaveBeenCalled()
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
})

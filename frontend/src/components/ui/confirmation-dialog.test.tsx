import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { ConfirmationDialog } from './confirmation-dialog'

describe('ConfirmationDialog', () => {
  it('renders content when open=true', () => {
    render(
      <ConfirmationDialog
        open={true}
        onStay={vi.fn()}
        onLeave={vi.fn()}
        reason="pending_questions"
      />
    )
    expect(screen.getByRole('alertdialog')).toBeInTheDocument()
    expect(screen.getByText('You have pending questions')).toBeInTheDocument()
  })

  it('is not visible when open=false', () => {
    render(
      <ConfirmationDialog
        open={false}
        onStay={vi.fn()}
        onLeave={vi.fn()}
        reason="pending_questions"
      />
    )
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
  })

  it('"Stay" button calls onStay', () => {
    const onStay = vi.fn()
    render(
      <ConfirmationDialog
        open={true}
        onStay={onStay}
        onLeave={vi.fn()}
        reason="pending_questions"
      />
    )
    fireEvent.click(screen.getByText('Stay'))
    expect(onStay).toHaveBeenCalledTimes(1)
  })

  it('"Leave" button calls onLeave', () => {
    const onLeave = vi.fn()
    render(
      <ConfirmationDialog
        open={true}
        onStay={vi.fn()}
        onLeave={onLeave}
        reason="pending_questions"
      />
    )
    fireEvent.click(screen.getByText('Leave'))
    expect(onLeave).toHaveBeenCalledTimes(1)
  })

  it('renders unsent_draft reason content', () => {
    render(
      <ConfirmationDialog
        open={true}
        onStay={vi.fn()}
        onLeave={vi.fn()}
        reason="unsent_draft"
      />
    )
    expect(screen.getByText('You have an unsent message')).toBeInTheDocument()
  })
})

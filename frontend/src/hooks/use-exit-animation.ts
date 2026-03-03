import { useState, useEffect } from 'react'

/**
 * Manages visibility lifecycle for animated open/close transitions.
 * Keeps the element mounted during the closing animation so CSS exit
 * animations can play before the DOM node is removed.
 *
 * Uses React's "adjust state during render" pattern to detect prop
 * transitions, and a timer-based effect for delayed unmount.
 */
export function useExitAnimation(open: boolean, duration = 150) {
  const [prevOpen, setPrevOpen] = useState(open)
  const [animatingOut, setAnimatingOut] = useState(false)

  // Detect open↔closed transitions during render (React-sanctioned pattern)
  if (open !== prevOpen) {
    setPrevOpen(open)
    if (open) {
      setAnimatingOut(false)
    } else {
      setAnimatingOut(true)
    }
  }

  // Timer to complete the exit animation
  useEffect(() => {
    if (animatingOut) {
      const timer = setTimeout(() => setAnimatingOut(false), duration)
      return () => clearTimeout(timer)
    }
  }, [animatingOut, duration])

  return {
    visible: open || animatingOut,
    closing: animatingOut,
    dataState: open ? 'open' as const : 'closed' as const,
  } as const
}

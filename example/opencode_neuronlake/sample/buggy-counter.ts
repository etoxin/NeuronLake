export type CounterState = {
  count: number
  history: number[]
}

export function increment(state: CounterState): CounterState {
  state.count = state.count + 1
  state.history.push(state.count)
  return state
}

export function reset(state: CounterState): CounterState {
  state.count = 0
  state.history = []
  return state
}

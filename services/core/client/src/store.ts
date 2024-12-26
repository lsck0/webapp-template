import { create } from "zustand";

type State = {
    counter: number;
    increment: () => void;
    decrement: () => void;
};

const useStore = create<State>(set => ({
    counter: 0,
    increment: () => set(state => ({ counter: state.counter + 1 })),
    decrement: () => set(state => ({ counter: state.counter - 1 })),
}));

export const useCounter = () =>
    useStore(state => {
        return {
            counter: state.counter,
            increment: state.increment,
            decrement: state.decrement,
        };
    });

import { createContext, useContext } from 'react';
import type { Shiba } from '../shiba';

export const ShibaContext = createContext<Shiba | null>(null);

export function useShiba(): Shiba {
    return useContext(ShibaContext)!;
}

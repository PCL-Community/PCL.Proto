import picCommand from '@/assets/icons/Impulse_Command_Block.gif'
import picGrass from '@/assets/icons/Grass_Block_JE7_BE6.png'
import type { gameVersionType } from '@/api/gameVersions';

type showIconType = 'command' | 'grass' | 'stone' | 'gold' | 'neoforge';
type showGameType = gameVersionType | 'fool';

export const gameInfoIcon: Record<showGameType, showIconType> = {
    'snapshot': 'command',
    'release': 'grass',
    'old_beta': 'stone',
    'old_alpha': 'stone',
    'fool': 'gold'
}

export const showIconPath: Record<showIconType, string> = {
    command: picCommand,
    grass: picGrass,
    stone: '',
    gold: '',
    neoforge: '',
}
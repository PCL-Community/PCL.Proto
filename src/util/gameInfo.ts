import picCommand from '@/assets/icons/Impulse_Command_Block.gif?url'
import picGrass from '@/assets/icons/Grass_Block_JE7_BE6.png?url'
import picCubblestone from '@/assets/icons/Cobblestone_JE5_BE3.png?url'
import picGold from '@/assets/icons/Block_of_Gold_JE6_BE3.png?url'
import picFabric from '@/assets/icons/Fabric.png?url'
import type { gameVersionType } from '@/api/gameVersions';

export type showIconType = 'command' | 'grass' | 'stone' | 'gold' | 'neoforge' | 'fabric';
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
    stone: picCubblestone,
    gold: picGold,
    neoforge: '',
    fabric: picFabric
}
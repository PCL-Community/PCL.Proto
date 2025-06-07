import picCommand from '@/assets/icons/Impulse_Command_Block.gif'
import picGrass from '@/assets/icons/Grass_Block_JE7_BE6.png'

type gameInfoTypes = 'command' | 'grass' | 'stone' | 'gold' | 'neoforge';
export const gameInfoRecord: Record<gameInfoTypes, string> = {
    command: picCommand,
    grass: picGrass,
    stone: '',
    gold: '',
    neoforge: '',
}
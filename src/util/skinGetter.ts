type AvatarInputType = 'url' | 'username' | 'uuid'
import SteveSkin from "/default-skin/Steve_(classic_texture)_JE6.png"

export default function getSkinUrl(src: string, type?: AvatarInputType) {
    switch (type) {
        case 'username':
            return `https://minotar.net/skin/${src}`
        case 'uuid':
            return `https://crafatar.com/skins/${src}`
        case 'url':
            return src
        default:
            return SteveSkin
    }
}
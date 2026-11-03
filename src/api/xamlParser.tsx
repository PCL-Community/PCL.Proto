import { h, type CSSProperties, type VNodeChild } from 'vue'
import type { Element } from 'xml-js'
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import PHint from '@/components/widget/PHint.vue'
import PListItem from '@/components/widget/PListItem.vue'
import PSvgIcon from '@/components/widget/PSvgIcon.vue'
import PPath from '@/components/widget/PPath.vue'
import { pclImagePath } from '@/api/pclAssets'
import { pclText } from '@/api/pclLocale'
import { showIconPath, type showIconType } from '@/types/gameInfo'
import {
  controlName,
  eventOf,
  homepageUrl,
  homepageText,
  supportedHomepageEvents,
  xamlColor,
} from '@/util/homepageXaml'

export type HomepageAction = (type: string, data: string) => void

const icons: Record<string, showIconType> = {
  CommandBlock: 'command',
  Cobblestone: 'stone',
  Grass: 'grass',
  Fabric: 'fabric',
  Neoforge: 'neoforge',
  NeoForge: 'neoforge',
  Anvil: 'forge',
  Quilt: 'quilt',
  Cleanroom: 'cleanroom',
  GrassPath: 'optifine',
  Egg: 'liteloader',
  LabyMod: 'laby-mod',
  GoldBlock: 'gold',
}
function image(value: unknown) {
  if (typeof value !== 'string') return undefined
  const builtin = /^pack:\/\/application:,,,\/images\/Blocks\/([\w]+)\.png$/i.exec(value)
  if (builtin && icons[builtin[1]!]) return showIconPath[icons[builtin[1]!]!]
  const packed = /^pack:\/\/application:,,,\/images\/([\w./-]+\.(?:png|jpe?g|gif|webp|ico))$/i.exec(
    value,
  )
  if (packed && !packed[1]!.split('/').includes('..'))
    return pclImagePath(value, import.meta.env.BASE_URL)
  return homepageUrl(value)
}
function length(value: unknown) {
  return value !== undefined && /^-?\d+(?:\.\d+)?$/.test(String(value)) ? `${value}px` : undefined
}
function thickness(value: unknown) {
  const parts = String(value ?? '')
    .split(',')
    .map((part) => length(part.trim()))
  if (parts.some((part) => !part)) return undefined
  if (parts.length === 1 || parts.length === 2) return parts.join(' ')
  if (parts.length === 4) return [parts[1], parts[2], parts[3], parts[0]].join(' ')
  return undefined
}
function resolveResource(value: unknown) {
  const text = String(value ?? '')
  const key = /^\{(?:DynamicResource|StaticResource) ([\w.]+)\}$/.exec(text)
  return key ? pclText(key[1]!) : text
}
function styleOf(node: Element): CSSProperties {
  const a = node.attributes || {}
  const align: Record<string, string> = {
    Left: 'start',
    Right: 'end',
    Center: 'center',
    Stretch: 'stretch',
    Top: 'start',
    Bottom: 'end',
  }
  const style: CSSProperties = {
    minWidth: 0,
    boxSizing: 'border-box',
    margin: thickness(a.Margin),
    padding: thickness(a.Padding),
    width: length(a.Width),
    height: length(a.Height),
    minHeight: length(a.MinHeight),
    maxWidth: length(a.MaxWidth),
    maxHeight: length(a.MaxHeight),
    color: xamlColor(a.Foreground),
    background: xamlColor(a.Background),
    fontSize: length(a.FontSize),
    fontWeight: a.FontWeight === 'Bold' ? 'bold' : undefined,
    fontStyle: a.FontStyle === 'Italic' ? 'italic' : undefined,
    alignSelf: align[String(a.VerticalAlignment)],
    justifySelf: align[String(a.HorizontalAlignment)],
    opacity: a.Opacity === undefined ? undefined : Math.max(0, Math.min(1, Number(a.Opacity))),
    pointerEvents: a.IsHitTestVisible === 'False' ? 'none' : undefined,
  }
  if (a.HorizontalAlignment === 'Center') {
    style.marginLeft = 'auto'
    style.marginRight = 'auto'
  }
  if (a.HorizontalAlignment === 'Right') style.marginLeft = 'auto'
  if (a.Visibility === 'Collapsed') style.display = 'none'
  if (a.Visibility === 'Hidden') style.visibility = 'hidden'
  for (const [property, target] of [
    ['Grid.Row', 'gridRow'],
    ['Grid.Column', 'gridColumn'],
  ] as const) {
    if (/^\d+$/.test(String(a[property]))) {
      const span = Number(a[`${property}Span`] || 1)
      style[target] =
        `${Number(a[property]) + 1} / span ${Number.isInteger(span) && span > 0 ? span : 1}`
    }
  }
  return style
}
function tracks(node: Element, axis: 'Column' | 'Row') {
  const definitions = node.elements?.find((child) => child.name === `Grid.${axis}Definitions`)
  return definitions?.elements
    ?.filter((child) => child.type === 'element')
    .map((child) => {
      const value = String(child.attributes?.[axis === 'Column' ? 'Width' : 'Height'] || '*')
      const star = /^(\d*(?:\.\d+)?)\*$/.exec(value)
      const track = star ? `${star[1] || 1}fr` : value === 'Auto' ? 'auto' : length(value) || 'auto'
      const min = length(child.attributes?.[axis === 'Column' ? 'MinWidth' : 'MinHeight'])
      return star ? `minmax(${min || '0px'}, ${track})` : track
    })
    .join(' ')
}

export function renderHomepageNodes(
  nodes: Element[],
  action?: HomepageAction,
  variables: Record<string, string> = {},
): VNodeChild[] {
  const resource = (value: unknown) => homepageText(resolveResource(value), variables)
  function render(node: Element): VNodeChild {
    if (node.type === 'text' || node.type === 'cdata')
      return homepageText(node.text ?? node.cdata, variables)
    if (node.type !== 'element') return null
    const name = controlName(node)
    if (name.includes('.'))
      return /\.(?:Children|Content|Inlines|Child)$/.test(name) ? node.elements?.map(render) : null
    const a = node.attributes || {}
    const event = eventOf(node)
    const hasEvent = Boolean(event.type && event.type !== 'None')
    const supported = !hasEvent || supportedHomepageEvents.has(event.type)
    const disabled = a.IsEnabled === 'False' || !supported
    const onClick =
      hasEvent && supported && !disabled && action
        ? (e: MouseEvent | KeyboardEvent) => {
            e.stopPropagation()
            action(event.type, resource(event.data))
          }
        : undefined
    const style = styleOf(node)
    const props = { style, title: resource(a.ToolTip), onClick, inert: disabled || undefined }
    const content = () => node.elements?.map(render)
    const text = () => (a.Text !== undefined ? resource(a.Text) : content())
    const svg = () =>
      a.SvgIcon
        ? h(PSvgIcon, { name: String(a.SvgIcon) })
        : a.Logo
          ? h(PPath, { data: String(a.Logo), style: { width: '20px', height: '20px' } })
          : null
    switch (name) {
      case 'MyCard':
        return h(
          PCard,
          {
            ...props,
            title: resource(a.Title),
            defaultFoldStatus: /^true$/i.test(String(a.CanSwap))
              ? /^true$/i.test(String(a.IsSwapped ?? a.IsSwaped))
                ? 'fold'
                : 'unfold'
              : 'unfoldable',
            key: `${a.CanSwap}:${a.IsSwapped ?? a.IsSwaped}`,
            foldArrow: a.SwapLogoRight === 'True' ? 'right' : 'down',
            hideTitle: !a.Title,
          },
          {
            default: () =>
              h(
                'div',
                { class: 'homepage-card-content' },
                node.elements?.map((child) => {
                  // CE overlays its title; PCard already places the title above the content.
                  const margin = String(child.attributes?.Margin || '').replaceAll(' ', '')
                  return render(
                    /^25,(?:38|40),23,15$/.test(margin)
                      ? { ...child, attributes: { ...child.attributes, Margin: '0' } }
                      : child,
                  )
                }),
              ),
          },
        )
      case 'MyHint':
        return h(
          PHint,
          {
            ...props,
            severity: a.Theme === 'Blue' ? 'info' : a.Theme === 'Yellow' ? 'warning' : 'error',
          },
          { default: text },
        )
      case 'MyButton':
      case 'MyTextButton':
      case 'MyIconTextButton':
      case 'MyIconButton':
        return h(
          PButton,
          {
            ...props,
            disabled,
            inline: name === 'MyTextButton',
            'aria-label': resource(a.ToolTip || a.Text || a.SvgIcon || 'Button'),
            type:
              a.ColorType === 'Highlight'
                ? 'tint'
                : a.ColorType === 'Red' || a.Theme === 'Red'
                  ? 'warn'
                  : 'default',
          },
          {
            default: () => [
              name.includes('Icon') ? svg() : null,
              name === 'MyIconButton' ? null : text(),
            ],
          },
        )
      case 'MyListItem':
        return h(PListItem, {
          ...props,
          title: resource(a.Title),
          subtitle: resource(a.Info),
          icon: image(resource(a.Logo)),
          clickable: Boolean(onClick),
          onClick,
        })
      case 'Image':
      case 'MyImage': {
        const source = image(resource(a.Source)) || image(resource(a.FallbackSource))
        if (!source)
          return h(
            'span',
            { ...props, role: 'img', 'aria-label': resource(a.ToolTip || a.Source) },
            resource(a.ToolTip || a.Source),
          )
        const fallback = image(resource(a.FallbackSource))
        return h('img', {
          ...props,
          src: source,
          alt: resource(a.ToolTip),
          loading: 'lazy',
          style: {
            maxWidth: '100%',
            objectFit: a.Stretch === 'Fill' ? 'fill' : 'contain',
            ...style,
          },
          onError: fallback
            ? (e: Event) => {
                const img = e.target as HTMLImageElement
                if (img.src !== new URL(fallback, img.ownerDocument.baseURI).href)
                  img.src = fallback
              }
            : undefined,
        })
      }
      case 'Path':
        return h(PPath, {
          ...props,
          data: String(a.Data || ''),
          style: { color: xamlColor(a.Fill), ...style },
        })
      case 'TextBlock':
        return h(
          'p',
          {
            ...props,
            style: { margin: 0, whiteSpace: 'pre-wrap', overflowWrap: 'anywhere', ...style },
          },
          text(),
        )
      case 'LineBreak':
        return h('br')
      case 'Run':
      case 'Span':
      case 'Bold':
      case 'Italic':
      case 'Underline':
        return h(
          ({ Bold: 'strong', Italic: 'em', Underline: 'u' } as Record<string, string>)[name] ||
            'span',
          props,
          text(),
        )
      case 'Hyperlink':
        return h(
          'a',
          {
            ...props,
            href: homepageUrl(resource(a.NavigateUri)),
            target: '_blank',
            rel: 'noopener noreferrer',
          },
          text(),
        )
      case 'StackPanel':
      case 'WrapPanel':
        return h(
          'div',
          {
            ...props,
            style: {
              display: 'flex',
              flexDirection:
                a.Orientation === 'Horizontal' || name === 'WrapPanel' ? 'row' : 'column',
              flexWrap: name === 'WrapPanel' ? 'wrap' : undefined,
              ...style,
            },
          },
          content(),
        )
      case 'Grid':
        return h(
          'div',
          {
            ...props,
            style: {
              display: 'grid',
              gridTemplateColumns: tracks(node, 'Column'),
              gridTemplateRows: tracks(node, 'Row'),
              ...style,
            },
          },
          content(),
        )
      case 'Border':
        return h(
          'div',
          {
            ...props,
            style: {
              borderStyle: 'solid',
              borderWidth: thickness(a.BorderThickness) || 0,
              borderColor: xamlColor(a.BorderBrush),
              borderRadius: thickness(a.CornerRadius),
              ...style,
            },
          },
          content(),
        )
      default:
        return null
    }
  }
  return nodes.map(render)
}

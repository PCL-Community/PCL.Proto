import CardInfoItem from '@/components/widget/CardInfoItem.vue'
import type { ButtonType } from '@/components/widget/PButton.vue'
import PButton from '@/components/widget/PButton.vue'
import type { FoldStatus } from '@/components/widget/PCard.vue'
import PCard from '@/components/widget/PCard.vue'
import type { Severity } from '@/components/widget/PHint.vue'
import PHint from '@/components/widget/PHint.vue'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import router from '@/router'
import { showIconPath, type showIconType } from '@/types/gameInfo'
import { type Component } from 'vue'
import { xml2js, type Element } from 'xml-js'

const modal = useModal()

const pageTypes = [
  'home',
  'download',
  'link',
  'setup',
  'more',
  'instance_select',
  'download_manager',
  'instance_setting',
  'comp_detail',
  'help_detail',
]
// const pageSubType = []

export default function renderFromXaml(xaml: string): Component {
  const page = xml2js(xaml, { compact: false })
  return page.elements?.map((el: any) => parseXamlElement(el))
}

function handleEvent(eventType: string, eventData: string) {
  let onClick: (() => void) | undefined = undefined
  switch (eventType) {
    case '打开网页':
      onClick = () => {
        window.open(eventData as string)
      }
      break
    case '弹出窗口':
      onClick = () => {
        modal.open({
          title: eventData.split('|')[0],
          content: eventData.split('|')[1],
        })
      }
      break
    case '切换页面':
      onClick = () => {
        let pageType = Number(eventData.split('|')[0])
        let pageSubType = Number(eventData.split('|')[1])
        router.push({ name: pageTypes[pageType] })
      }
      break
    case '启动游戏':
    case '执行命令':
    case '今日人品':
      // TODO))
      break
    case '复制文本':
      onClick = () => {
        navigator.clipboard.writeText(eventData).then(() => {
          sideTip.show('已成功复制！', 'success')
        })
      }
      break
  }
  return onClick
}

function handleIcon(sourceRaw: string) {
  const iconMap: Record<string, showIconType> = {
    'Blocks/CommandBlock.png': 'command',
    'Blocks/Cobblestone.png': 'stone',
    'Blocks/Grass.png': 'grass',
    'Blocks/Fabric.png': 'fabric',
    'Blocks/Neoforge.png': 'neoforge',
  }
  if (sourceRaw.startsWith('pack://application:,,,/images/')) {
    let iconPath = sourceRaw.split('/images/')[1] as string
    console.log('[xaml] ', iconPath)
    return showIconPath[iconMap[iconPath] as showIconType]
  } else {
    return sourceRaw
  }
}

function parseXamlElement(element: Element): Component | string | null {
  if (!element || element.type != 'element') return null
  if (element.name?.startsWith('local:')) {
    const localType = element.name.split(':')[1]
    // 处理事件
    let onClick: (() => void) | undefined = undefined
    if (element.attributes?.EventType) {
      onClick = handleEvent(
        element.attributes?.EventType as string,
        element.attributes?.EventData as string,
      )
    }
    // console.log('[xaml] got local-element:', localType)
    switch (localType) {
      case 'MyCard':
        // console.log('[xaml] got MyCard', element)
        let foldStatus: FoldStatus = 'unfoldable'
        const attributes = element.attributes! as {
          CanSwap: 'True' | 'False'
          IsSwaped: 'True' | 'False'
          Margin: string
          Title: string
        }
        if (attributes.CanSwap == 'True') {
          foldStatus = attributes.IsSwaped == 'True' ? 'fold' : 'unfold'
        }
        return (
          <PCard defaultFoldStatus={foldStatus} title={attributes.Title}>
            {element.elements?.map((el) => parseXamlElement(el))}
          </PCard>
        )
      case 'MyHint':
        let severity: Severity = 'error'
        if (element.attributes?.Theme == 'Blue') severity = 'info'
        if (element.attributes?.Theme == 'Red') severity = 'error'
        return <PHint severity={severity}>{element.attributes?.Text || element.text}</PHint>
      case 'MyButton':
        let type: ButtonType = 'default'
        if (element.attributes?.ColorType == 'Highlight') type = 'tint'
        if (element.attributes?.ColorType == 'Red') type = 'warn'
        return (
          <PButton
            type={type}
            tooltip={element.attributes?.ToolTip as string | undefined}
            click={onClick}
          >
            {element.attributes?.Text}
          </PButton>
        )
      case 'MyListItem':
        return (
          <CardInfoItem
            title={element.attributes?.Title as string}
            subtitle={element.attributes?.Info as string}
            isGameInfo={false}
            icon={handleIcon(element.attributes?.Logo as string)}
            click={onClick}
          />
        )
      case 'MyImage':
        return handleIcon(element.attributes?.Source as string) // TODO
      default:
        return JSON.stringify(element)
    }
  } else {
    switch (element.name) {
      case 'StackPanel':
        return (
          <section style={{ display: 'flex', flexDirection: 'column' }}>
            {element.elements?.map((el) => parseXamlElement(el))}
          </section>
        )
      case 'TextBlock':
        return (
          <p
            style={{
              fontSize: element.attributes?.FontSize + 'px',
              color: element.attributes?.Foreground as string,
            }}
          >
            {element.attributes?.Text || element.text}
          </p>
        )
      case 'Path':
        console.log(element)
        return (
          <svg
            width={element.attributes!.Width}
            height={element.attributes!.Height}
            viewBox="0 0 1024 1024"
          >
            <path d={element.attributes!.Data as string}></path>
          </svg>
        )
      default:
        return JSON.stringify(element)
    }
  }
}

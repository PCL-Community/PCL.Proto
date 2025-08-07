import CardInfoItem from "@/components/widget/CardInfoItem.vue"
import type { ButtonType } from "@/components/widget/PButton.vue"
import PButton from "@/components/widget/PButton.vue"
import type { FoldStatus } from "@/components/widget/PCard.vue"
import PCard from "@/components/widget/PCard.vue"
import type { Severity } from "@/components/widget/PHint.vue"
import PHint from "@/components/widget/PHint.vue"
import sideTip from "@/composables/sideTip"
import { useModal } from "@/composables/useModal"
import router from "@/router"
import { h, type VNode, type VNodeTypes } from "vue"
import { xml2js, type Element } from "xml-js"
const modal = useModal()

const pageTypes = ['home', 'download', 'link', 'setup', 'more', 'instance_select', 'download_manager', 'instance_setting', 'comp_detail', 'help_detail']
// const pageSubType = []

export default function renderFromXaml(xaml: string): VNodeTypes {
  const page = xml2js(xaml, { compact: false })
  return page.elements?.map((el: any) => parseXamlElement(el))
}

function parseXamlElement(element: Element): VNode | string | (VNode | string)[] | null {
  if (!element || element.type != 'element') return null
  if (element.name?.startsWith('local:')) {
    const localType = element.name.split(':')[1]
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
        return h(
          PCard,
          { defaultFoldStatus: foldStatus },
          { title: () => attributes.Title, content: () => element.elements?.map((el) => parseXamlElement(el)) },
        )
      case 'MyHint':
        let severity: Severity = 'error'
        if (element.attributes?.Theme == 'Blue') severity = 'info'
        if (element.attributes?.Theme == 'Red') severity = 'error'
        return h(PHint, { severity }, () => element.attributes?.Text || element.text)
      case 'MyButton':
        let type: ButtonType = 'default'
        if (element.attributes?.ColorType == 'Highlight') type = 'tint'
        if (element.attributes?.ColorType == 'Red') type = 'warn'
        // console.log('[xaml] got button:', element)
        let onClick: () => void
        let eventData = element.attributes?.EventData as string
        switch (element.attributes?.EventType) {
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
        return h(PButton, { type, tooltip: element.attributes?.ToolTip as string | undefined, click: () => onClick() }, () => element.attributes?.Text)
      case 'MyListItem':
        return h(CardInfoItem, {
          title: element.attributes?.Title as string,
          subtitle: element.attributes?.Info as string,
          isGameInfo: false,
          icon: element.attributes?.Logo as string
        })
      default:
        return JSON.stringify(element)
    }
  } else {
    switch (element.name) {
      case 'StackPanel':
        return h(
          'section',
          { style: { display: "flex", flexDirection: 'column' } },
          element.elements?.map((el) => parseXamlElement(el)),
        )
      case 'TextBlock':
        return h(
          'p',
          {
            style: {
              fontSize: element.attributes?.FontSize + 'px',
              color: element.attributes?.Foreground,
            },
          },
          element.attributes?.Text || element.text,
        )
      default:
        return JSON.stringify(element)
    }
  }
}
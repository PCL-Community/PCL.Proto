import type { FoldStatus } from "@/components/widget/MyCard.vue"
import MyCard from "@/components/widget/MyCard.vue"
import type { Severity } from "@/components/widget/MyHint.vue"
import MyHint from "@/components/widget/MyHint.vue"
import { h, type VNode } from "vue"
import { xml2js, type Element } from "xml-js"

export default function renderFromXaml(xaml: string): (VNode | string)[] {
    const page = xml2js(xaml, { compact: false })
    return page.elements?.map((el: any) => parseXamlElement(el))
}

function parseXamlElement(element: Element): VNode | string {
    if (element.name?.startsWith('local:')) {
        const localType = element.name.split(':')[1]
        console.log('[xaml] got local-element:', localType)
        switch (localType) {
            case 'MyCard':
                console.log('[xaml] got MyCard', element)
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
                    MyCard,
                    { defaultFoldStatus: foldStatus },
                    { title: attributes.Title, content: element.elements?.map((el) => parseXamlElement(el)) },
                )
            case 'MyHint':
                let severity = 'info' as Severity
                if (element.attributes?.Theme == 'Yellow') severity = 'warning'
                if (element.attributes?.Theme == 'Red') severity = 'error'
                return h(MyHint, { severity }, element.attributes?.Text || element.text)
            default:
                return JSON.stringify(element)
        }
    } else {
        switch (element.name) {
            case 'StackPanel':
                return h(
                    'section',
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
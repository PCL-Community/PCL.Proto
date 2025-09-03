<script lang="tsx">
import PButton from '@/components/widget/PButton.vue'
import PCard from '@/components/widget/PCard.vue'
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'LinkHelp',
  setup() {
    const cardContents = [
      [
        '加入大厅超时怎么办？',
        '请先检查联机设置中的 EasyTier 设置。在节点设置中双方应当有至少一个相同的节点以确保能够被相互发现。',
      ],
      [
        '启动 EasyTier 失败怎么办？',
        '请检查 EasyTier 是否被杀毒软件拦截，网络是否可以正常连接到数据服务器。有问题可以加入用户群，或是前往 GitHub 反馈。',
      ],
      ['联机可以用服务端么？', '可以的，Minecraft 服务端也可以用于联机。'],
      [
        '是否需要实名认证？',
        '目前无此要求，在联机设置中填入用户名或登录 Natayark 账号即可联机，如有变动，请以公告提示与实际情况为准。',
      ],
      [
        '跨启动器联机行不行？',
        <>
          截至目前，PCL CE 已经与 HMCL（陶瓦）实现基础的互联功能，你可以直接与使用
          HMCL（陶瓦）的朋友们共享大厅编号并联机。
          <br />
          另外，PCL CE 正在与 HTML 协商并共同开发
          Scaffolding（暂名）跨启动器联机协议，未来将支持更多拓展功能。
        </>,
      ],
      [
        'NAT 类型是什么，对联机有影响么？',
        <>
          <p>
            简单来说：会，但是在使用大厅的过程中影响不大。如果你想进一步了解，可以继续阅读下面的简要介绍。
          </p>
          <p>
            NAT 是计算机网络中的一个概念，它会将多个内网IP地址映射为一个公网IP地址。更为宽松的 NAT
            环境可以提高点对点连接成功率，这可以使联机延迟更低。一般来说，绝大部分中国大陆家庭宽带的
            NAT 类型均为端口受限圆锥或对称。
          </p>
          <p>你还可以阅读来自 Tailscale 的介绍文章（英文）以对 NAT 有更深入的了解。</p>
          <PButton click={() => open('https://tailscale.com/blog/how-nat-traversal-works')}>
            打开介绍文章
          </PButton>
        </>,
      ],
    ]
    return () =>
      cardContents.map((content) => (
        <PCard>{{ title: () => content[0], content: () => content[1] }}</PCard>
      ))
  },
})
</script>

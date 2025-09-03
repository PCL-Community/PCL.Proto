import renderFromXaml from '@/api/xamlParser'
import customXaml from '@/assets/homepage/Custom.xaml?raw'

export default {
    setup() {
        return () => renderFromXaml(customXaml)
    },
}

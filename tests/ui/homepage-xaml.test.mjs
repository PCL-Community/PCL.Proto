import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import {
  parseHomepage,
  serializeHomepage,
  eventOf,
  controlName,
  homepageUrl,
  homepageLimit,
  xamlColor,
  homepageRoute,
  homepageText,
} from '../../src/util/homepageXaml.ts'

const fragment = `<!-- CE fragments need no namespace declarations -->
<local:MyCard Title="A &amp; B" CanSwap="True">
  <StackPanel Margin="25,40,23,15">
    <TextBlock>Inline <Bold>bold</Bold><LineBreak />text</TextBlock>
    <Grid><Grid.ColumnDefinitions><ColumnDefinition Width="*"/><ColumnDefinition Width="2*"/></Grid.ColumnDefinitions>
      <local:MyButton Grid.Column="1" local:CustomEventService.EventType="OpenUrl" local:CustomEventService.EventData="https://example.com/?a=1&amp;b=2" Text="Go" />
    </Grid>
  </StackPanel>
</local:MyCard>`
const parsed = parseHomepage(fragment)
assert.deepEqual(parsed.warnings, [])
const roundtrip = parseHomepage(serializeHomepage(parsed.nodes))
assert.equal(roundtrip.nodes.find((node) => node.type === 'element').attributes.Title, 'A & B')
const button = parsed.nodes
  .find((node) => node.type === 'element')
  .elements.find((node) => node.type === 'element')
  .elements.find((node) => node.name === 'Grid')
  .elements.find((node) => node.name === 'local:MyButton')
assert.deepEqual(eventOf(button), { type: 'OpenUrl', data: 'https://example.com/?a=1&b=2' })
assert.deepEqual(
  eventOf(
    parseHomepage('<local:MyButton EventType="打开网页" EventData="https://example.com/"/>')
      .nodes[0],
  ),
  { type: 'OpenUrl', data: 'https://example.com/' },
)
assert.deepEqual(
  eventOf(
    parseHomepage(
      '<local:MyButton><local:CustomEventService.EventType>CopyText</local:CustomEventService.EventType><local:CustomEventService.EventData>Hello</local:CustomEventService.EventData></local:MyButton>',
    ).nodes[0],
  ),
  { type: 'CopyText', data: 'Hello' },
)
assert.equal(controlName({ name: 'local:PCard' }), 'MyCard')
assert.equal(homepageUrl('javascript:alert(1)'), undefined)
assert.equal(homepageUrl('file:///etc/passwd'), undefined)
assert.equal(homepageUrl('https://user:secret@example.com/'), undefined)
assert.equal(homepageUrl(undefined), undefined)
assert.equal(xamlColor('#80445566'), '#44556680')
assert.equal(xamlColor('{DynamicResource ColorBrush2}'), 'var(--ce-color-2)')
assert.equal(xamlColor('url(https://example.com/tracker)'), undefined)
assert.throws(() => parseHomepage('<TextBlock>'))
assert.throws(() => parseHomepage('<!DOCTYPE x [<!ENTITY a "test">]><TextBlock/>'))
assert.throws(() => parseHomepage('a'.repeat(homepageLimit + 1)))
assert.throws(() => parseHomepage('<StackPanel>'.repeat(66) + '</StackPanel>'.repeat(66)))
assert.throws(() => parseHomepage('<TextBlock/>'.repeat(5001)))
assert.deepEqual(
  parseHomepage('<ObjectDataProvider/><local:MyButton EventType="ExecuteCommand"/>').warnings,
  ['ObjectDataProvider', 'EventType: ExecuteCommand'],
)
assert.deepEqual(
  parseHomepage(
    await readFile(new URL('../../src/assets/homepage/Custom.xaml', import.meta.url), 'utf8'),
  ).warnings,
  [],
)
console.log(
  'CE fragments, attached events, XML round trips, URL/color safety and complexity limits passed.',
)

assert.equal(homepageRoute('2|1'), '/setup/personalization')
assert.equal(homepageRoute('Setup|SetupJava'), '/setup/java')
assert.equal(homepageRoute('download|DownloadFabric'), '/download/fabric')
assert.equal(homepageRoute('7|6'), '/instance_setting/mod/disabled')
assert.equal(homepageRoute('InstanceSetup|VersionInstall'), '/instance_setting/modify')
assert.throws(() => homepageRoute('8|1'))
assert.throws(() => homepageRoute('Setup|Unknown'))

assert.equal(
  homepageText('Hello {user}: {name}', { user: 'A & <B>', name: '1.21' }),
  'Hello A & <B>: 1.21',
)
assert.deepEqual(parseHomepage('<TextBlock Text="{user}"/>').warnings, [])
const special = 'Quotes " &amp; & < >\nnext'
const specialNode = { type: 'element', name: 'TextBlock', attributes: { Text: special } }
assert.equal(parseHomepage(serializeHomepage([specialNode])).nodes[0].attributes.Text, special)

const literalText = parseHomepage('<TextBlock>A &amp;amp; B &amp; C &lt; D</TextBlock>')
assert.deepEqual(parseHomepage(serializeHomepage(literalText.nodes)), literalText)

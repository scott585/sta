// React and Semantic UI 元素。
import React, { useState, useEffect } from 'react';
import { Form, Input, Grid, Message } from 'semantic-ui-react';
// 预设的 Substrate front-end 工具，用于连接节点
// 和执行交易。
import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';
// Polkadot-JS 工具，用作对数据进行 hash 计算 。
import { blake2AsHex } from '@polkadot/util-crypto';

// 导出存证的主要组件。
export function Main (props) {
  // 建立一个与 Substrate 节点通讯的 API。
  const { api } = useSubstrate();
  // 从 `AccountSelector` 组件中获取选定的用户。
  const { accountPair } = props;
  // React hooks 用于追踪所有变量的状态。
  // 更多内容参见：https://reactjs.org/docs/hooks-intro.html
  const [status, setStatus] = useState('');
  const [digest, setDigest] = useState('');
  const [owner, setOwner] = useState('');
  const [block, setBlock] = useState(0);

  // 被后续代码中的函数所访问的 `FileReader()` 实例。
  let fileReader;

  // 使用 Blake2 256 散列函数生成文件的摘要。
  const bufferToDigest = () => {
    // 将文件内容转换成 Hex 格式。
    const content = Array.from(new Uint8Array(fileReader.result))
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('');

    const hash = blake2AsHex(content, 256);
    setDigest(hash);
  };

  // 当一个新文件被选取时需调用的回调函数。
  const handleFileChosen = (file) => {
    fileReader = new FileReader();
    fileReader.onloadend = bufferToDigest;
    fileReader.readAsArrayBuffer(file);
  };

  // 使用 React hook 来更新文件中的 owner 和  block number 信息。
  useEffect(() => {
    let unsubscribe;

    // 使用 Polkadot-JS API 来查询 pallet 中  `proofs` 的存储信息。
    // 这是一个订阅，它将总是获得最新的值，
    // 即便它发生了改变。
    api.query.templateModule
      .proofs(digest, (result) => {
        // 我们的存储项将返回一个元组，以一个数组来代表。
        setOwner(result[0].toString());
        setBlock(result[1].toNumber());
      })
      .then((unsub) => {
        unsubscribe = unsub;
      });

    return () => unsubscribe && unsubscribe();
    // 用来告诉 React hook 在文件的摘要发生更改
    // (当新文件被选择时)，或当存储订阅表示存储项的值已更新时，
    // 执行页面更新操作。
  }, [digest, api.query.templateModule]);

  // 若储存文件摘要的区块值不为零，我们则称该文件摘要已被声明。
  function isClaimed () {
    return block !== 0;
  }

  // 从组件中返回实际的 UI 元素。
  return (
    <Grid.Column>
      <h1>Proof Of Existence</h1>
      {/* 当文件被声明或失败时，显示警告或成功的消息。 */}
      <Form success={!!digest && !isClaimed()} warning={isClaimed()}>
        <Form.Field>
          {/* 回调函数为 `handleFileChosen` 的文件选择器。 */}
          <Input
            type='file'
            id='file'
            label='Your File'
            onChange={ e => handleFileChosen(e.target.files[0]) }
          />
          {/* 如果要声明的文件可用，则显示此消息 */}
          <Message success header='File Digest Unclaimed' content={digest} />
          {/* 如果文件已被声明，则显示此消息。 */}
          <Message
            warning
            header='File Digest Claimed'
            list={[digest, `Owner: ${owner}`, `Block: ${block}`]}
          />
        </Form.Field>
        {/* 与组件交互的 Buttons。 */}
        <Form.Field>
          {/* 用于创建声明的 Button。 只有在文件被选择且所选文件未被声明时，
          可用。 更新 `status`。 */}
          <TxButton
            accountPair={accountPair}
            label={'Create Claim'}
            setStatus={setStatus}
            type='SIGNED-TX'
            disabled={isClaimed() || !digest}
            attrs={{
              palletRpc: 'templateModule',
              callable: 'createClaim',
              inputParams: [digest],
              paramFields: [true]
            }}
          />
          {/* 用于撤销声明的 Button。 只有在文件被选择且所选文件已被声明时，
          可用。 更新 `status`。 */}
          <TxButton
            accountPair={accountPair}
            label='Revoke Claim'
            setStatus={setStatus}
            type='SIGNED-TX'
            disabled={!isClaimed() || owner !== accountPair.address}
            attrs={{
              palletRpc: 'templateModule',
              callable: 'revokeClaim',
              inputParams: [digest],
              paramFields: [true]
            }}
          />
        </Form.Field>
        {/* 交易的状态信息。 */}
        <div style={{ overflowWrap: 'break-word' }}>{status}</div>
      </Form>
    </Grid.Column>
  );
}

export default function TemplateModule (props) {
  const { api } = useSubstrate();
  return (api.query.templateModule && api.query.templateModule.proofs
    ? <Main {...props} /> : null);
}
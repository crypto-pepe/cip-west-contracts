import { invoke } from '../scripts/transaction';
import { NetworkConfig } from '../scripts/network';
import { ProofsGenerator } from '../scripts/script';
import { getAddressFromPrivateKey } from '../scripts/helpers';
import { Keypair } from '@wavesenterprise/signer';

export default async function (
  deployerSeed: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator
) {
  const deployerPrivateKey = await (
    await Keypair.fromExistingSeedPhrase(deployerSeed)
  ).privateKey();
  const deployerAddress = await getAddressFromPrivateKey(
    deployerPrivateKey,
    network.chainID
  );

  let wavesCallerContractAddress = '';
  let tokenWavesAdapterContractAddress = '';
  switch (network.name) {
    case 'mainnet':
      wavesCallerContractAddress = '';
      tokenWavesAdapterContractAddress = ''; // TODO
      throw 'todo';
      break;
    case 'testnet':
      wavesCallerContractAddress =
        '59aJQqPYsH5nkVnXrqR7DhHDV91kmVhdvQXofQn72wmX';
      tokenWavesAdapterContractAddress =
        'E5wD1qGQzqsTwycz8n7VCYtqBvVFZDAzsZcH9TH68rAS';
      break;
  }

  await invoke(
    {
      contractId: wavesCallerContractAddress,
      contractVersion: 1,
      callFunction: 'allow',
      callParams: [
        {
          type: 'string',
          key: 'caller',
          value: tokenWavesAdapterContractAddress,
        },
      ],
      callPayments: [],
    },
    deployerPrivateKey,
    network,
    proofsGenerator
  ).catch((e) => {
    throw e;
  });

  return true;
}

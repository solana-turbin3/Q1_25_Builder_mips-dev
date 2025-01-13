import bs58 from 'bs58';
import prompt from 'prompt-sync';

// Initialize prompt-sync
const input = prompt();

// Function to convert base58 to wallet byte array
function base58ToWallet() {
  const base58Key = input('Enter your base58-encoded private key: ');
  try {
    const walletBytes = bs58.decode(base58Key);
    console.log('Wallet Byte Array:', Array.from(walletBytes));
  } catch (error) {
    console.error('Invalid base58 string:', error);
  }
}

// Function to convert wallet byte array to base58
function walletToBase58() {
  const walletBytesInput = input(
    'Enter your wallet byte array (comma-separated numbers): '
  );
  try {
    const walletBytes = Uint8Array.from(
      walletBytesInput.split(',').map((num) => parseInt(num.trim()))
    );
    const base58Key = bs58.encode(walletBytes);
    console.log('Base58 Encoded Private Key:', base58Key);
  } catch (error) {
    console.error('Invalid wallet byte array:', error);
  }
}

// Main CLI menu
console.log('Select an option:');
console.log('1: Convert Base58 to Wallet');
console.log('2: Convert Wallet to Base58');
const option = input('Your choice: ');

if (option === '1') {
  base58ToWallet();
} else if (option === '2') {
  walletToBase58();
} else {
  console.log('Invalid option selected.');
}


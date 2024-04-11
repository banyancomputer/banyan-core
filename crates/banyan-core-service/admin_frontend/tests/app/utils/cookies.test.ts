import { getSessionKey, getLocalKey, destroyLocalKey } from '@/app/utils/cookies';
import { setCookie, destroyCookie } from 'nookies';

describe('Cookies utility functions', () => {
  afterEach(() => {
    jest.resetAllMocks();
  });

  test('getSessionKey should return session key from cookies', () => {
    const mockSessionKey = 'mockSessionKey';
    setCookie(null, '_session_id', mockSessionKey);
    expect(getSessionKey()).toBe(mockSessionKey);
    destroyCookie(null, '_session_id');
  });

  test('getLocalKey should return local key from cookies', () => {
    const mockId = 'mockId';
    const mockKey = 'mockKey';
    setCookie(null, '_local_key', `${mockId}:${mockKey}`);
    const localKey = getLocalKey();
    expect(localKey.id).toBe(mockId);
    expect(localKey.key).toBe(mockKey);
    destroyCookie(null, '_local_key');
  });

  test('destroyLocalKey should remove local key from cookies', () => {
    const mockId = 'mockId';
    const mockKey = 'mockKey';
    setCookie(null, '_local_key', `${mockId}:${mockKey}`);
    destroyLocalKey();
    const newLocalKey = getLocalKey();
    expect(newLocalKey.id).not.toBe(mockId);
    expect(newLocalKey.key).not.toBe(mockKey);
  });
});

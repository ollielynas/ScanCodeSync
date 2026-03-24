export const getOrCreateDeviceId = () => {
  const match = document.cookie.match(/(?:^|;\s*)deviceId=(\d+)/);
  if (match) return parseInt(match[1], 10);
  const id = Math.floor(Math.random() * 0xffff);
  document.cookie = `deviceId=${id}; max-age=${10 * 365 * 24 * 60 * 60}; SameSite=Strict`;
  return id;
};

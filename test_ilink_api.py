#!/usr/bin/env python3
"""
iLink API 测试 Demo
参考 /data/weixin-ClawBot-API/weixin-bot-api.md
"""

import base64
import random
import json
import sys
import urllib.request
import urllib.error

BASE_URL = "https://ilinkai.weixin.qq.com"


def make_headers(token=None):
    """生成请求头"""
    uin = str(random.randint(0, 0xFFFFFFFF))
    headers = {
        "Content-Type": "application/json",
        "AuthorizationType": "ilink_bot_token",
        "X-WECHAT-UIN": base64.b64encode(uin.encode()).decode(),
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers


def test_get_qrcode():
    """测试获取 QR 码"""
    print("=" * 60)
    print("测试 1: 获取 QR 码")
    print(f"URL: {BASE_URL}/ilink/bot/get_bot_qrcode?bot_type=3")
    print("=" * 60)

    url = f"{BASE_URL}/ilink/bot/get_bot_qrcode?bot_type=3"
    req = urllib.request.Request(url, headers=make_headers(), method='GET')

    try:
        with urllib.request.urlopen(req, timeout=30) as res:
            print(f"HTTP 状态码: {res.status}")
            text = res.read().decode('utf-8')
            print(f"响应内容: {text[:500]}")

            if res.status == 200:
                try:
                    data = json.loads(text)
                    qrcode = data.get("qrcode")
                    qrcode_url = data.get("qrcode_img_content", "")
                    print(f"\n✅ 成功获取 QR 码!")
                    print(f"qrcode: {qrcode}")
                    print(f"qrcode_url: {qrcode_url[:200] if qrcode_url else 'N/A'}")
                    return qrcode
                except json.JSONDecodeError as e:
                    print(f"❌ JSON 解析失败: {e}")
                    return None
            else:
                print(f"❌ 请求失败: HTTP {res.status}")
                return None
    except urllib.error.HTTPError as e:
        print(f"❌ HTTP 错误: {e.code}")
        print(f"响应内容: {e.read().decode('utf-8')[:500]}")
        return None
    except Exception as e:
        print(f"❌ 请求异常: {e}")
        return None


def test_qrcode_status(qrcode):
    """测试查询 QR 码状态"""
    if not qrcode:
        print("\n跳过状态查询（没有 qrcode）")
        return

    print("\n" + "=" * 60)
    print("测试 2: 查询 QR 码状态")
    print(f"URL: {BASE_URL}/ilink/bot/get_qrcode_status?qrcode={qrcode[:20]}...")
    print("=" * 60)

    url = f"{BASE_URL}/ilink/bot/get_qrcode_status?qrcode={qrcode}"
    req = urllib.request.Request(url, headers=make_headers(), method='GET')

    try:
        with urllib.request.urlopen(req, timeout=30) as res:
            print(f"HTTP 状态码: {res.status}")
            text = res.read().decode('utf-8')
            print(f"响应内容: {text[:500]}")

            if res.status == 200:
                try:
                    data = json.loads(text)
                    status = data.get("status")
                    print(f"\n当前状态: {status}")
                    if status == "confirmed":
                        print(f"bot_token: {data.get('bot_token', 'N/A')[:50]}...")
                        print(f"baseurl: {data.get('baseurl', 'N/A')}")
                except json.JSONDecodeError as e:
                    print(f"❌ JSON 解析失败: {e}")
            else:
                print(f"❌ 请求失败: HTTP {res.status}")
    except urllib.error.HTTPError as e:
        print(f"❌ HTTP 错误: {e.code}")
        print(f"响应内容: {e.read().decode('utf-8')[:500]}")
    except Exception as e:
        print(f"❌ 请求异常: {e}")


def main():
    print("\n" + "=" * 60)
    print("iLink API 测试工具")
    print("参考: /data/weixin-ClawBot-API/weixin-bot-api.md")
    print("=" * 60 + "\n")

    # 测试 1: 获取 QR 码
    qrcode = test_get_qrcode()

    if qrcode:
        # 测试 2: 查询状态（只查一次，不循环等待）
        test_qrcode_status(qrcode)

        print("\n" + "=" * 60)
        print("📱 请使用微信扫描以下二维码完成登录:")
        print("=" * 60)
        print(f"QR Code: {qrcode}")
        print("\n提示: 如果上面没有显示二维码链接，说明 API 响应格式可能有变化")
    else:
        print("\n❌ 获取 QR 码失败，请检查:")
        print("1. 网络是否能访问 ilinkai.weixin.qq.com")
        print("2. API 端点是否正确")
        print("3. 请求头是否完整")
        sys.exit(1)


if __name__ == "__main__":
    main()

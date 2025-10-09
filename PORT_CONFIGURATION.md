# 端口配置说明

## 问题解决

由于80端口经常被其他服务占用，本项目已配置为使用**8080端口**。

## 访问地址

启动服务后，请访问：

```
前端: http://localhost:8080
后端: http://localhost:8000
API文档: http://localhost:8080/api-docs
健康检查: http://localhost:8080/health
```

## 端口配置

当前配置在 `docker-compose.yml` 中：

```yaml
nginx:
  ports:
    - "8080:80"    # HTTP
    - "8443:443"   # HTTPS
```

## 修改端口

如果8080端口也被占用，可以修改为其他端口：

1. 编辑 `docker-compose.yml`：
```yaml
nginx:
  ports:
    - "9090:80"    # 改为9090或其他可用端口
    - "9443:443"
```

2. 重启服务：
```bash
docker compose down
docker compose up -d
```

3. 访问新地址：
```
http://localhost:9090
```

## 生产环境

在生产环境中使用自己的域名时：

1. 配置域名的Nginx配置文件
2. 可以使用标准的80/443端口
3. 配置SSL证书
4. 参考 `nginx/conf.d/README.template.conf`

## 常见端口冲突

如果遇到端口占用错误：

```
Error: bind: address already in use
```

解决方法：
1. 查看哪个进程占用端口：`lsof -i :8080`
2. 停止该进程或更改本项目的端口配置
3. 使用上述步骤修改端口

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  profile = "default"
  region  = "eu-central-1"
}

resource "aws_s3_bucket" "webapp-template-bucket" {
  bucket        = "webapp-template-bucket"
  force_destroy = true

  tags = {
    Project = "webapp-template"
    Owner   = "Luca Sandrock"
  }
}

resource "aws_elastic_beanstalk_application" "webapp-template" {
  name        = "webapp-template"
  description = "Webapp Template Application"

  tags = {
    Project = "webapp-template"
    Owner   = "Luca Sandrock"
  }
}

resource "aws_elastic_beanstalk_environment" "webapp-template-env" {
  name                = "webapp-template-env"
  application         = aws_elastic_beanstalk_application.webapp-template.name
  solution_stack_name = "64bit Amazon Linux 2023 v4.6.0 running Docker"
  cname_prefix        = "webapp-template"

  tags = {
    Project = "webapp-template"
    Owner   = "Luca Sandrock"
  }

  # EC2 Base Instance
  # Name        | vCPU | Memory (GiB) | Cost in $/h | Cost in $/month
  # ----------- | ---- | ------------ | ----------- | ---------------
  # t3.nano     | 2    | 0.5          | 0.0052      | 3.744            <-- Only usable if monitoring tools are disabled
  # t3.micro    | 2    | 1            | 0.0104      | 7.488
  # t3.small    | 2    | 2            | 0.0208      | 14.97
  # t3.medium   | 2    | 4            | 0.0417      | 30.048           <-- Minimal instance for the Full Webapp Template (Monitoring Tools are heavy on RAM)
  # t3.large    | 2    | 8            | 0.0832      | 60.096
  # t3.xlarge   | 4    | 16           | 0.1664      | 120.192
  # t3.2xlarge  | 8    | 32           | 0.3328      | 240.384
  setting {
    namespace = "aws:autoscaling:launchconfiguration"
    name      = "InstanceType"
    value     = "t3.medium"
  }

  # Single Instance Environment aka no Load Balancer or Auto Scaling
  setting {
    namespace = "aws:elasticbeanstalk:environment"
    name      = "EnvironmentType"
    value     = "SingleInstance"
  }

  # Disk Space
  setting {
    namespace = "aws:autoscaling:launchconfiguration"
    name      = "RootVolumeSize"
    value     = "16" # GiB
  }

  # EC2 Instance Role
  setting {
    namespace = "aws:autoscaling:launchconfiguration"
    name      = "IamInstanceProfile"
    value     = "aws-elasticbeanstalk-ec2-role"
  }

  # EC2 Key Pair
  setting {
    namespace = "aws:autoscaling:launchconfiguration"
    name      = "EC2KeyName"
    value     = "lsandrock"
  }

  # EC2 Security Group
  setting {
    namespace = "aws:autoscaling:launchconfiguration"
    name      = "SecurityGroups"
    value     = aws_security_group.webapp-template-sg.name
  }

  # Enhanced Health Reporting
  setting {
    namespace = "aws:elasticbeanstalk:healthreporting:system"
    name      = "SystemType"
    value     = "enhanced"
  }

  # Longer Command Timeout
  setting {
    namespace = "aws:elasticbeanstalk:command"
    name      = "Timeout"
    value     = "3600" # 60 minutes
  }
}

resource "aws_security_group" "webapp-template-sg" {
  name   = "webapp-template-sg"
  vpc_id = "vpc-8a4974e1"

  tags = {
    Project = "webapp-template"
    Owner   = "Luca Sandrock"
  }
}

resource "aws_vpc_security_group_ingress_rule" "webapp-template-sg-http-inbound" {
  security_group_id = aws_security_group.webapp-template-sg.id
  description       = "Allow HTTP inbound traffic to the main application."

  ip_protocol = "tcp"
  cidr_ipv4   = "0.0.0.0/0"
  from_port   = 80
  to_port     = 80

  tags = {
    Project = "webapp-template"
    Owner   = "Luca Sandrock"
  }
}

resource "aws_vpc_security_group_ingress_rule" "webapp-template-sg-ssh-inbound" {
  security_group_id = aws_security_group.webapp-template-sg.id
  description       = "Allow SSH inbound traffic only from Capgemini PCs."

  ip_protocol = "tcp"
  cidr_ipv4   = "147.161.234.0/24"
  from_port   = 22
  to_port     = 22

  tags = {
    Project = "webapp-template"
    Owner   = "Luca Sandrock"
  }
}

resource "aws_vpc_security_group_ingress_rule" "webapp-template-sg-monitoring-inbound" {
  security_group_id = aws_security_group.webapp-template-sg.id
  description       = "Allow HTTP inbound traffic to the monitoring tools only from Capgemini PCs."

  ip_protocol = "tcp"
  cidr_ipv4   = "147.161.234.0/24"
  from_port   = 3000
  to_port     = 3002

  tags = {
    Project = "webapp-template"
    Owner   = "Luca Sandrock"
  }
}
